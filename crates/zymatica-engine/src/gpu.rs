use crate::tensor::Matrix;
use anyhow::{Context, Result, bail};
use std::borrow::Cow;
use std::collections::HashMap;
use std::sync::{Arc, Mutex, mpsc};
use wgpu::util::DeviceExt;

const WGPU_MATVEC_SHADER: &str = r#"
struct Params {
    rows: u32,
    cols: u32,
    row_dispatch_width: u32,
    pad1: u32,
};

@group(0) @binding(0)
var<storage, read> matrix: array<vec4<f32>>;

@group(0) @binding(1)
var<storage, read> x: array<vec4<f32>>;

@group(0) @binding(2)
var<storage, read_write> output: array<f32>;

@group(0) @binding(3)
var<uniform> params: Params;

var<workgroup> partial_sums: array<f32, 64>;

@compute @workgroup_size(64)
fn main(
    @builtin(workgroup_id) workgroup_id: vec3<u32>,
    @builtin(local_invocation_id) local_id: vec3<u32>,
) {
    let row = workgroup_id.x + workgroup_id.z * params.row_dispatch_width;
    let batch = workgroup_id.y;
    if (row >= params.rows) {
        return;
    }

    var sum = 0.0;
    for (var c = local_id.x; c < params.cols; c = c + 64u) {
        sum = sum + dot(matrix[row * params.cols + c], x[batch * params.cols + c]);
    }
    partial_sums[local_id.x] = sum;
    workgroupBarrier();

    var stride = 32u;
    loop {
        if (stride == 0u) {
            break;
        }
        if (local_id.x < stride) {
            partial_sums[local_id.x] = partial_sums[local_id.x]
                + partial_sums[local_id.x + stride];
        }
        workgroupBarrier();
        stride = stride / 2u;
    }
    if (local_id.x == 0u) {
        output[batch * params.rows + row] = partial_sums[0];
    }
}
"#;

const WGPU_Q3_MATVEC_SHADER: &str = r#"
struct Params {
    rows: u32,
    cols: u32,
    packed_cols: u32,
    output_row_base: u32,
    row_dispatch_width: u32,
    pad0: u32,
    pad1: u32,
    pad2: u32,
};

@group(0) @binding(0)
var<storage, read> packed_weights: array<u32>;

@group(0) @binding(1)
var<storage, read> scales: array<f32>;

@group(0) @binding(2)
var<storage, read> x: array<f32>;

@group(0) @binding(3)
var<storage, read_write> output: array<f32>;

@group(0) @binding(4)
var<uniform> params: Params;

var<workgroup> partial_sums: array<f32, 64>;

fn load_3bytes(byte_index: u32) -> u32 {
    let word_idx = byte_index >> 2u;
    let shift = (byte_index & 3u) << 3u;
    let w0 = packed_weights[word_idx];
    let w1 = select(packed_weights[word_idx + 1u], 0u, shift == 0u);
    return select((w0 >> shift) | (w1 << (32u - shift)), w0 >> shift, shift == 0u);
}

fn signed_q3(code: u32) -> f32 {
    return f32(i32(code) - 4);
}

@compute @workgroup_size(64)
fn main(
    @builtin(workgroup_id) workgroup_id: vec3<u32>,
    @builtin(local_invocation_id) local_id: vec3<u32>,
) {
    let row = workgroup_id.x + workgroup_id.z * params.row_dispatch_width;
    if (row >= params.rows) {
        return;
    }

    var sum = 0.0;
    let groups = (params.cols + 7u) / 8u;
    for (var group = local_id.x; group < groups; group = group + 64u) {
        let col = group * 8u;
        let byte_index = row * params.packed_cols + group * 3u;
        let bytes = load_3bytes(byte_index);
        let b0 = bytes & 255u;
        let b1 = (bytes >> 8u) & 255u;
        let b2 = (bytes >> 16u) & 255u;

        if (col < params.cols) {
            sum = sum + signed_q3(b0 & 7u) * x[col];
        }
        if (col + 1u < params.cols) {
            sum = sum + signed_q3((b0 >> 3u) & 7u) * x[col + 1u];
        }
        if (col + 2u < params.cols) {
            let code = ((b0 >> 6u) & 3u) | ((b1 & 1u) << 2u);
            sum = sum + signed_q3(code) * x[col + 2u];
        }
        if (col + 3u < params.cols) {
            sum = sum + signed_q3((b1 >> 1u) & 7u) * x[col + 3u];
        }
        if (col + 4u < params.cols) {
            sum = sum + signed_q3((b1 >> 4u) & 7u) * x[col + 4u];
        }
        if (col + 5u < params.cols) {
            let code = ((b1 >> 7u) & 1u) | ((b2 & 3u) << 1u);
            sum = sum + signed_q3(code) * x[col + 5u];
        }
        if (col + 6u < params.cols) {
            sum = sum + signed_q3((b2 >> 2u) & 7u) * x[col + 6u];
        }
        if (col + 7u < params.cols) {
            sum = sum + signed_q3((b2 >> 5u) & 7u) * x[col + 7u];
        }
    }
    partial_sums[local_id.x] = sum;
    workgroupBarrier();

    var stride = 32u;
    loop {
        if (stride == 0u) {
            break;
        }
        if (local_id.x < stride) {
            partial_sums[local_id.x] = partial_sums[local_id.x]
                + partial_sums[local_id.x + stride];
        }
        workgroupBarrier();
        stride = stride / 2u;
    }
    if (local_id.x == 0u) {
        output[params.output_row_base + row] = partial_sums[0] * scales[row];
    }
}
"#;

const WGPU_Q3_ACTIVATION_SHADER: &str = r#"
struct Params {
    len: u32,
    activation: u32,
    pad0: u32,
    pad1: u32,
};

@group(0) @binding(0)
var<storage, read_write> gate: array<f32>;

@group(0) @binding(1)
var<storage, read> up: array<f32>;

@group(0) @binding(2)
var<uniform> params: Params;

fn gelu_pytorch_tanh(x: f32) -> f32 {
    let inner = 0.7978846 * (x + 0.044715 * x * x * x);
    return 0.5 * x * (1.0 + tanh(inner));
}

fn silu(x: f32) -> f32 {
    return x / (1.0 + exp(-x));
}

@compute @workgroup_size(256)
fn main(@builtin(global_invocation_id) global_id: vec3<u32>) {
    let index = global_id.x;
    if (index >= params.len) {
        return;
    }
    let value = gate[index];
    if (params.activation == 1u) {
        gate[index] = gelu_pytorch_tanh(value) * up[index];
    } else {
        gate[index] = silu(value) * up[index];
    }
}
"#;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GpuBackendInfo {
    pub adapter_name: String,
    pub backend: String,
    pub device_type: String,
}

struct WgpuContext {
    device: wgpu::Device,
    queue: wgpu::Queue,
    pipeline: wgpu::ComputePipeline,
    q3_pipeline: wgpu::ComputePipeline,
    q3_activation_pipeline: wgpu::ComputePipeline,
    info: GpuBackendInfo,
    limits: wgpu::Limits,
}

#[derive(Clone)]
pub struct WgpuMatvecBackend {
    context: Arc<WgpuContext>,
}

pub struct WgpuMatvecPlan {
    context: Arc<WgpuContext>,
    _matrix_buffer: wgpu::Buffer,
    _params_buffer: wgpu::Buffer,
    x_buffer: wgpu::Buffer,
    output_buffer: wgpu::Buffer,
    readback_buffer: wgpu::Buffer,
    bind_group: wgpu::BindGroup,
    rows: usize,
    cols: usize,
    padded_cols: usize,
    max_batch: usize,
    dispatch_x: u32,
    dispatch_z: u32,
}

pub struct Q3MatrixUpload<'a> {
    pub key: usize,
    pub rows: usize,
    pub cols: usize,
    pub scales: &'a [f32],
    pub packed: &'a [u8],
}

struct WgpuQ3Chunk {
    _packed_buffer: wgpu::Buffer,
    _scales_buffer: wgpu::Buffer,
    _params_buffer: wgpu::Buffer,
    bind_group: wgpu::BindGroup,
    dispatch_x: u32,
    dispatch_z: u32,
}

struct WgpuQ3MatrixPlan {
    rows: usize,
    cols: usize,
    output_buffer: wgpu::Buffer,
    chunks: Vec<WgpuQ3Chunk>,
}

struct WgpuQ3MlpPlan {
    _activation_params: wgpu::Buffer,
    activation_bind_group: wgpu::BindGroup,
    activation_dispatch_x: u32,
    down_bind_groups: Vec<wgpu::BindGroup>,
}

pub struct WgpuQ3ModelRuntime {
    context: Arc<WgpuContext>,
    matrices: HashMap<usize, WgpuQ3MatrixPlan>,
    mlp_plans: HashMap<(usize, usize, usize), WgpuQ3MlpPlan>,
    x_buffer: wgpu::Buffer,
    readback_buffer: wgpu::Buffer,
    max_cols: usize,
    max_fused_rows: usize,
    resident_bytes: u64,
    execution_lock: Mutex<()>,
}

impl std::fmt::Debug for WgpuQ3ModelRuntime {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("WgpuQ3ModelRuntime")
            .field("adapter", &self.context.info.adapter_name)
            .field("matrices", &self.matrices.len())
            .field("mlp_plans", &self.mlp_plans.len())
            .field("max_cols", &self.max_cols)
            .field("max_fused_rows", &self.max_fused_rows)
            .field("resident_bytes", &self.resident_bytes)
            .finish()
    }
}

impl WgpuMatvecBackend {
    pub fn new() -> Result<Self> {
        pollster::block_on(Self::new_async())
    }

    async fn new_async() -> Result<Self> {
        let instance = wgpu::Instance::new(wgpu::InstanceDescriptor::new_without_display_handle());
        let adapter = instance
            .request_adapter(&wgpu::RequestAdapterOptions {
                power_preference: wgpu::PowerPreference::HighPerformance,
                compatible_surface: None,
                force_fallback_adapter: false,
                ..Default::default()
            })
            .await
            .context("requesting a wgpu adapter")?;
        let adapter_info = adapter.get_info();
        let (device, queue) = adapter
            .request_device(&wgpu::DeviceDescriptor {
                label: Some("zymatica-gpu-device"),
                required_features: wgpu::Features::empty(),
                required_limits: wgpu::Limits::downlevel_defaults(),
                memory_hints: wgpu::MemoryHints::Performance,
                experimental_features: wgpu::ExperimentalFeatures::disabled(),
                trace: wgpu::Trace::Off,
            })
            .await
            .context("requesting a wgpu device")?;
        let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("zymatica-gpu-matvec-shader"),
            source: wgpu::ShaderSource::Wgsl(WGPU_MATVEC_SHADER.into()),
        });
        let pipeline = device.create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
            label: Some("zymatica-gpu-matvec-pipeline"),
            layout: None,
            module: &shader,
            entry_point: Some("main"),
            compilation_options: wgpu::PipelineCompilationOptions::default(),
            cache: None,
        });
        let q3_shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("zymatica-gpu-q3-matvec-shader"),
            source: wgpu::ShaderSource::Wgsl(WGPU_Q3_MATVEC_SHADER.into()),
        });
        let q3_pipeline = device.create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
            label: Some("zymatica-gpu-q3-matvec-pipeline"),
            layout: None,
            module: &q3_shader,
            entry_point: Some("main"),
            compilation_options: wgpu::PipelineCompilationOptions::default(),
            cache: None,
        });
        let q3_activation_shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("zymatica-gpu-q3-activation-shader"),
            source: wgpu::ShaderSource::Wgsl(WGPU_Q3_ACTIVATION_SHADER.into()),
        });
        let q3_activation_pipeline =
            device.create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
                label: Some("zymatica-gpu-q3-activation-pipeline"),
                layout: None,
                module: &q3_activation_shader,
                entry_point: Some("main"),
                compilation_options: wgpu::PipelineCompilationOptions::default(),
                cache: None,
            });
        let limits = device.limits();
        Ok(Self {
            context: Arc::new(WgpuContext {
                device,
                queue,
                pipeline,
                q3_pipeline,
                q3_activation_pipeline,
                info: GpuBackendInfo {
                    adapter_name: adapter_info.name,
                    backend: format!("{:?}", adapter_info.backend),
                    device_type: format!("{:?}", adapter_info.device_type),
                },
                limits,
            }),
        })
    }

    pub fn info(&self) -> &GpuBackendInfo {
        &self.context.info
    }

    pub fn matvec(&self, matrix: &Matrix, x: &[f32]) -> Result<Vec<f32>> {
        if matrix.cols != x.len() {
            bail!(
                "matvec dimension mismatch: matrix cols={} x len={}",
                matrix.cols,
                x.len()
            );
        }
        if matrix.rows == 0 || matrix.cols == 0 {
            return Ok(vec![0.0; matrix.rows]);
        }
        let mut plan = self.prepare_matrix(matrix, 1)?;
        plan.matvec(x)
    }

    pub fn prepare_matrix(&self, matrix: &Matrix, max_batch: usize) -> Result<WgpuMatvecPlan> {
        if matrix.rows == 0 || matrix.cols == 0 {
            bail!("GPU resident matrices must have non-zero dimensions");
        }
        if max_batch == 0 {
            bail!("GPU matvec max_batch must be at least one");
        }
        let rows_u32 = u32::try_from(matrix.rows).context("GPU matrix rows exceed u32")?;
        let max_dispatch = self.context.limits.max_compute_workgroups_per_dimension;
        if u32::try_from(max_batch).context("GPU max_batch exceeds u32")? > max_dispatch {
            bail!("GPU max_batch {max_batch} exceeds device dispatch limit {max_dispatch}");
        }
        let dispatch_x = rows_u32.min(max_dispatch);
        let dispatch_z = rows_u32.div_ceil(dispatch_x);
        if dispatch_z > max_dispatch {
            bail!(
                "GPU matrix needs {dispatch_z} row tiles, exceeding device dispatch limit {max_dispatch}"
            );
        }
        let vector_cols = matrix.cols.div_ceil(4);
        let vector_cols_u32 = u32::try_from(vector_cols).context("GPU matrix cols exceed u32")?;
        let padded_cols = vector_cols
            .checked_mul(4)
            .context("overflow padding GPU matrix columns")?;
        let matrix_data: Cow<'_, [f32]> = if padded_cols == matrix.cols {
            Cow::Borrowed(&matrix.data)
        } else {
            let padded_len = matrix
                .rows
                .checked_mul(padded_cols)
                .context("overflow padding GPU matrix")?;
            let mut padded = vec![0.0_f32; padded_len];
            for row in 0..matrix.rows {
                let src_start = row * matrix.cols;
                let dst_start = row * padded_cols;
                padded[dst_start..dst_start + matrix.cols]
                    .copy_from_slice(&matrix.data[src_start..src_start + matrix.cols]);
            }
            Cow::Owned(padded)
        };
        let x_elements = max_batch
            .checked_mul(padded_cols)
            .context("overflow allocating GPU activation buffer")?;
        let output_elements = max_batch
            .checked_mul(matrix.rows)
            .context("overflow allocating GPU output buffer")?;
        let x_bytes = buffer_bytes(x_elements, "GPU activation buffer")?;
        let output_bytes = buffer_bytes(output_elements, "GPU output buffer")?;
        validate_storage_buffer(
            buffer_bytes(matrix_data.len(), "GPU matrix buffer")?,
            &self.context.limits,
            "GPU matrix buffer",
        )?;
        validate_storage_buffer(x_bytes, &self.context.limits, "GPU activation buffer")?;
        validate_storage_buffer(output_bytes, &self.context.limits, "GPU output buffer")?;
        let matrix_buffer =
            self.context
                .device
                .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                    label: Some("zymatica-gpu-matrix"),
                    contents: bytemuck::cast_slice(matrix_data.as_ref()),
                    usage: wgpu::BufferUsages::STORAGE,
                });
        let x_buffer = self.context.device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("zymatica-gpu-vector-batch"),
            size: x_bytes,
            usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });
        let output_buffer = self.context.device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("zymatica-gpu-output"),
            size: output_bytes,
            usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_SRC,
            mapped_at_creation: false,
        });
        let params = [rows_u32, vector_cols_u32, dispatch_x, 0];
        let params_buffer =
            self.context
                .device
                .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                    label: Some("zymatica-gpu-params"),
                    contents: bytemuck::cast_slice(&params),
                    usage: wgpu::BufferUsages::UNIFORM,
                });
        let readback_buffer = self.context.device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("zymatica-gpu-readback"),
            size: output_bytes,
            usage: wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::MAP_READ,
            mapped_at_creation: false,
        });

        let bind_group_layout = self.context.pipeline.get_bind_group_layout(0);
        let bind_group = self
            .context
            .device
            .create_bind_group(&wgpu::BindGroupDescriptor {
                label: Some("zymatica-gpu-matvec-bind-group"),
                layout: &bind_group_layout,
                entries: &[
                    wgpu::BindGroupEntry {
                        binding: 0,
                        resource: matrix_buffer.as_entire_binding(),
                    },
                    wgpu::BindGroupEntry {
                        binding: 1,
                        resource: x_buffer.as_entire_binding(),
                    },
                    wgpu::BindGroupEntry {
                        binding: 2,
                        resource: output_buffer.as_entire_binding(),
                    },
                    wgpu::BindGroupEntry {
                        binding: 3,
                        resource: params_buffer.as_entire_binding(),
                    },
                ],
            });

        Ok(WgpuMatvecPlan {
            context: Arc::clone(&self.context),
            _matrix_buffer: matrix_buffer,
            _params_buffer: params_buffer,
            x_buffer,
            output_buffer,
            readback_buffer,
            bind_group,
            rows: matrix.rows,
            cols: matrix.cols,
            padded_cols,
            max_batch,
            dispatch_x,
            dispatch_z,
        })
    }

    pub fn prepare_q3_model(&self, uploads: &[Q3MatrixUpload<'_>]) -> Result<WgpuQ3ModelRuntime> {
        if uploads.is_empty() {
            bail!("cannot prepare an empty Q3 GPU model");
        }
        let max_cols = uploads.iter().map(|matrix| matrix.cols).max().unwrap_or(0);
        if max_cols == 0 {
            bail!("Q3 GPU model contains a zero-width matrix");
        }
        let mut row_counts = uploads.iter().map(|matrix| matrix.rows).collect::<Vec<_>>();
        row_counts.sort_unstable_by(|a, b| b.cmp(a));
        let max_fused_rows = row_counts
            .into_iter()
            .take(3)
            .try_fold(0_usize, |total, rows| {
                total
                    .checked_add(rows)
                    .context("overflow sizing Q3 GPU fused output")
            })?;
        let x_bytes = buffer_bytes(max_cols, "Q3 GPU activation buffer")?;
        let readback_bytes = buffer_bytes(max_fused_rows, "Q3 GPU readback buffer")?;
        validate_storage_buffer(x_bytes, &self.context.limits, "Q3 GPU activation buffer")?;
        if readback_bytes > self.context.limits.max_buffer_size {
            bail!(
                "Q3 GPU readback requires {readback_bytes} bytes, exceeding device max_buffer_size {}",
                self.context.limits.max_buffer_size
            );
        }
        let x_buffer = self.context.device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("zymatica-gpu-q3-activation"),
            size: x_bytes,
            usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });
        let readback_buffer = self.context.device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("zymatica-gpu-q3-readback"),
            size: readback_bytes,
            usage: wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::MAP_READ,
            mapped_at_creation: false,
        });
        let bind_group_layout = self.context.q3_pipeline.get_bind_group_layout(0);
        let limits = &self.context.limits;
        let max_binding_bytes = limits
            .max_buffer_size
            .min(limits.max_storage_buffer_binding_size);
        let max_dispatch = limits.max_compute_workgroups_per_dimension;
        let max_dispatch_rows = u64::from(max_dispatch)
            .checked_mul(u64::from(max_dispatch))
            .context("overflow computing Q3 GPU dispatch capacity")?;
        let mut resident_bytes = x_bytes
            .checked_add(readback_bytes)
            .context("overflow accounting Q3 GPU resident buffers")?;
        let mut matrices = HashMap::with_capacity(uploads.len());

        for matrix in uploads {
            if matrix.rows == 0 || matrix.cols == 0 {
                bail!("Q3 GPU matrix {} has zero dimensions", matrix.key);
            }
            if matrix.scales.len() != matrix.rows {
                bail!(
                    "Q3 GPU matrix {} has {} scales for {} rows",
                    matrix.key,
                    matrix.scales.len(),
                    matrix.rows
                );
            }
            let packed_cols = matrix
                .cols
                .checked_mul(3)
                .context("overflow sizing Q3 GPU packed row")?
                .div_ceil(8);
            let expected_packed = matrix
                .rows
                .checked_mul(packed_cols)
                .context("overflow sizing Q3 GPU packed matrix")?;
            if matrix.packed.len() != expected_packed {
                bail!(
                    "Q3 GPU matrix {} packed length mismatch: expected {expected_packed}, got {}",
                    matrix.key,
                    matrix.packed.len()
                );
            }
            let output_bytes = buffer_bytes(matrix.rows, "Q3 GPU matrix output")?;
            validate_storage_buffer(output_bytes, limits, "Q3 GPU matrix output")?;
            let output_buffer = self.context.device.create_buffer(&wgpu::BufferDescriptor {
                label: Some("zymatica-gpu-q3-output"),
                size: output_bytes,
                usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_SRC,
                mapped_at_creation: false,
            });
            resident_bytes = resident_bytes
                .checked_add(output_bytes)
                .context("overflow accounting Q3 GPU output buffers")?;

            let rows_by_packed = max_binding_bytes.saturating_sub(3) / packed_cols as u64;
            let rows_by_scales = max_binding_bytes / std::mem::size_of::<f32>() as u64;
            let rows_per_chunk_u64 = rows_by_packed
                .min(rows_by_scales)
                .min(max_dispatch_rows)
                .min(matrix.rows as u64);
            let rows_per_chunk = usize::try_from(rows_per_chunk_u64)
                .context("Q3 GPU chunk row count exceeds usize")?;
            if rows_per_chunk == 0 {
                bail!(
                    "Q3 GPU packed row requires {packed_cols} bytes, exceeding device binding limit {max_binding_bytes}"
                );
            }

            let mut chunks = Vec::new();
            for row_start in (0..matrix.rows).step_by(rows_per_chunk) {
                let row_end = (row_start + rows_per_chunk).min(matrix.rows);
                let chunk_rows = row_end - row_start;
                let packed_start = row_start * packed_cols;
                let packed_end = row_end * packed_cols;
                let packed_words = pack_bytes_to_words(&matrix.packed[packed_start..packed_end]);
                let packed_bytes = buffer_bytes(packed_words.len(), "Q3 GPU packed chunk")?;
                validate_storage_buffer(packed_bytes, limits, "Q3 GPU packed chunk")?;
                let scales_bytes = buffer_bytes(chunk_rows, "Q3 GPU scale chunk")?;
                validate_storage_buffer(scales_bytes, limits, "Q3 GPU scale chunk")?;
                let packed_buffer =
                    self.context
                        .device
                        .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                            label: Some("zymatica-gpu-q3-packed"),
                            contents: bytemuck::cast_slice(&packed_words),
                            usage: wgpu::BufferUsages::STORAGE,
                        });
                let scales_buffer =
                    self.context
                        .device
                        .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                            label: Some("zymatica-gpu-q3-scales"),
                            contents: bytemuck::cast_slice(&matrix.scales[row_start..row_end]),
                            usage: wgpu::BufferUsages::STORAGE,
                        });
                let chunk_rows_u32 =
                    u32::try_from(chunk_rows).context("Q3 GPU chunk rows exceed u32")?;
                let dispatch_x = chunk_rows_u32.min(max_dispatch);
                let dispatch_z = chunk_rows_u32.div_ceil(dispatch_x);
                let params = [
                    chunk_rows_u32,
                    u32::try_from(matrix.cols).context("Q3 GPU cols exceed u32")?,
                    u32::try_from(packed_cols).context("Q3 GPU packed cols exceed u32")?,
                    u32::try_from(row_start).context("Q3 GPU output row offset exceeds u32")?,
                    dispatch_x,
                    0,
                    0,
                    0,
                ];
                let params_buffer =
                    self.context
                        .device
                        .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                            label: Some("zymatica-gpu-q3-params"),
                            contents: bytemuck::cast_slice(&params),
                            usage: wgpu::BufferUsages::UNIFORM,
                        });
                let bind_group =
                    self.context
                        .device
                        .create_bind_group(&wgpu::BindGroupDescriptor {
                            label: Some("zymatica-gpu-q3-bind-group"),
                            layout: &bind_group_layout,
                            entries: &[
                                wgpu::BindGroupEntry {
                                    binding: 0,
                                    resource: packed_buffer.as_entire_binding(),
                                },
                                wgpu::BindGroupEntry {
                                    binding: 1,
                                    resource: scales_buffer.as_entire_binding(),
                                },
                                wgpu::BindGroupEntry {
                                    binding: 2,
                                    resource: x_buffer.as_entire_binding(),
                                },
                                wgpu::BindGroupEntry {
                                    binding: 3,
                                    resource: output_buffer.as_entire_binding(),
                                },
                                wgpu::BindGroupEntry {
                                    binding: 4,
                                    resource: params_buffer.as_entire_binding(),
                                },
                            ],
                        });
                resident_bytes = resident_bytes
                    .checked_add(packed_bytes)
                    .and_then(|bytes| bytes.checked_add(scales_bytes))
                    .and_then(|bytes| bytes.checked_add(params.len() as u64 * 4))
                    .context("overflow accounting Q3 GPU matrix chunks")?;
                chunks.push(WgpuQ3Chunk {
                    _packed_buffer: packed_buffer,
                    _scales_buffer: scales_buffer,
                    _params_buffer: params_buffer,
                    bind_group,
                    dispatch_x,
                    dispatch_z,
                });
            }

            let old = matrices.insert(
                matrix.key,
                WgpuQ3MatrixPlan {
                    rows: matrix.rows,
                    cols: matrix.cols,
                    output_buffer,
                    chunks,
                },
            );
            if old.is_some() {
                bail!("duplicate Q3 GPU matrix key {}", matrix.key);
            }
        }

        Ok(WgpuQ3ModelRuntime {
            context: Arc::clone(&self.context),
            matrices,
            mlp_plans: HashMap::new(),
            x_buffer,
            readback_buffer,
            max_cols,
            max_fused_rows,
            resident_bytes,
            execution_lock: Mutex::new(()),
        })
    }
}

impl WgpuMatvecPlan {
    pub fn matvec(&mut self, x: &[f32]) -> Result<Vec<f32>> {
        let mut outputs = self.matvec_batch(&[x])?;
        Ok(outputs.pop().expect("single-item GPU batch returned empty"))
    }

    pub fn matvec_batch(&mut self, inputs: &[&[f32]]) -> Result<Vec<Vec<f32>>> {
        if inputs.is_empty() {
            bail!("GPU matvec batch must contain at least one vector");
        }
        if inputs.len() > self.max_batch {
            bail!(
                "GPU matvec batch size {} exceeds prepared maximum {}",
                inputs.len(),
                self.max_batch
            );
        }
        for (index, input) in inputs.iter().enumerate() {
            if input.len() != self.cols {
                bail!(
                    "GPU matvec input {index} has length {}, expected {}",
                    input.len(),
                    self.cols
                );
            }
        }

        let input_data: Cow<'_, [f32]> = if inputs.len() == 1 && self.padded_cols == self.cols {
            Cow::Borrowed(inputs[0])
        } else {
            let input_len = inputs
                .len()
                .checked_mul(self.padded_cols)
                .context("overflow padding GPU activation batch")?;
            let mut padded = vec![0.0_f32; input_len];
            for (batch, input) in inputs.iter().enumerate() {
                let start = batch * self.padded_cols;
                padded[start..start + self.cols].copy_from_slice(input);
            }
            Cow::Owned(padded)
        };
        self.context.queue.write_buffer(
            &self.x_buffer,
            0,
            bytemuck::cast_slice(input_data.as_ref()),
        );

        let output_elements = inputs
            .len()
            .checked_mul(self.rows)
            .context("overflow reading GPU output batch")?;
        let output_bytes = buffer_bytes(output_elements, "GPU output readback")?;
        let batch_u32 = u32::try_from(inputs.len()).context("GPU batch size exceeds u32")?;
        let mut encoder =
            self.context
                .device
                .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                    label: Some("zymatica-gpu-matvec-encoder"),
                });
        {
            let mut pass = encoder.begin_compute_pass(&wgpu::ComputePassDescriptor {
                label: Some("zymatica-gpu-matvec-pass"),
                timestamp_writes: None,
            });
            pass.set_pipeline(&self.context.pipeline);
            pass.set_bind_group(0, &self.bind_group, &[]);
            pass.dispatch_workgroups(self.dispatch_x, batch_u32, self.dispatch_z);
        }
        encoder.copy_buffer_to_buffer(
            &self.output_buffer,
            0,
            &self.readback_buffer,
            0,
            output_bytes,
        );
        self.context.queue.submit([encoder.finish()]);

        let slice = self.readback_buffer.slice(..output_bytes);
        let (sender, receiver) = mpsc::channel();
        slice.map_async(wgpu::MapMode::Read, move |result| {
            let _ = sender.send(result);
        });
        self.context
            .device
            .poll(wgpu::PollType::wait_indefinitely())
            .context("polling wgpu device")?;
        receiver
            .recv()
            .context("waiting for wgpu readback")?
            .context("mapping wgpu readback buffer")?;
        let view = slice
            .get_mapped_range()
            .context("reading mapped wgpu output range")?;
        let flat_output = bytemuck::cast_slice::<u8, f32>(&view).to_vec();
        drop(view);
        self.readback_buffer.unmap();
        Ok(flat_output
            .chunks_exact(self.rows)
            .map(<[f32]>::to_vec)
            .collect())
    }
}

impl WgpuQ3ModelRuntime {
    pub fn info(&self) -> &GpuBackendInfo {
        &self.context.info
    }

    pub fn matrix_count(&self) -> usize {
        self.matrices.len()
    }

    pub fn resident_bytes(&self) -> u64 {
        self.resident_bytes
    }

    pub fn mlp_plan_count(&self) -> usize {
        self.mlp_plans.len()
    }

    pub fn prepare_mlp_plans(
        &mut self,
        triples: &[(usize, usize, usize)],
        hidden_activation: &str,
    ) -> Result<()> {
        let activation = match hidden_activation {
            "gelu_pytorch_tanh" | "gelu_fast" | "gelu_approx_tanh" => 1_u32,
            _ => 0_u32,
        };
        let activation_layout = self.context.q3_activation_pipeline.get_bind_group_layout(0);
        let q3_layout = self.context.q3_pipeline.get_bind_group_layout(0);
        let max_dispatch = self.context.limits.max_compute_workgroups_per_dimension;

        for &(gate_key, up_key, down_key) in triples {
            let plan = {
                let gate = self
                    .matrices
                    .get(&gate_key)
                    .with_context(|| format!("Q3 GPU MLP gate matrix {gate_key} is missing"))?;
                let up = self
                    .matrices
                    .get(&up_key)
                    .with_context(|| format!("Q3 GPU MLP up matrix {up_key} is missing"))?;
                let down = self
                    .matrices
                    .get(&down_key)
                    .with_context(|| format!("Q3 GPU MLP down matrix {down_key} is missing"))?;
                if gate.rows != up.rows || gate.cols != up.cols {
                    bail!(
                        "Q3 GPU MLP gate/up shape mismatch: {}x{} versus {}x{}",
                        gate.rows,
                        gate.cols,
                        up.rows,
                        up.cols
                    );
                }
                if down.cols != gate.rows {
                    bail!(
                        "Q3 GPU MLP down input {} does not match intermediate width {}",
                        down.cols,
                        gate.rows
                    );
                }
                let activation_dispatch_x = u32::try_from(gate.rows.div_ceil(256))
                    .context("Q3 GPU MLP activation dispatch exceeds u32")?;
                if activation_dispatch_x > max_dispatch {
                    bail!(
                        "Q3 GPU MLP activation dispatch {activation_dispatch_x} exceeds device limit {max_dispatch}"
                    );
                }
                let params = [
                    u32::try_from(gate.rows).context("Q3 GPU MLP width exceeds u32")?,
                    activation,
                    0,
                    0,
                ];
                let activation_params =
                    self.context
                        .device
                        .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                            label: Some("zymatica-gpu-q3-mlp-activation-params"),
                            contents: bytemuck::cast_slice(&params),
                            usage: wgpu::BufferUsages::UNIFORM,
                        });
                let activation_bind_group =
                    self.context
                        .device
                        .create_bind_group(&wgpu::BindGroupDescriptor {
                            label: Some("zymatica-gpu-q3-mlp-activation-bind-group"),
                            layout: &activation_layout,
                            entries: &[
                                wgpu::BindGroupEntry {
                                    binding: 0,
                                    resource: gate.output_buffer.as_entire_binding(),
                                },
                                wgpu::BindGroupEntry {
                                    binding: 1,
                                    resource: up.output_buffer.as_entire_binding(),
                                },
                                wgpu::BindGroupEntry {
                                    binding: 2,
                                    resource: activation_params.as_entire_binding(),
                                },
                            ],
                        });
                let down_bind_groups = down
                    .chunks
                    .iter()
                    .map(|chunk| {
                        self.context
                            .device
                            .create_bind_group(&wgpu::BindGroupDescriptor {
                                label: Some("zymatica-gpu-q3-mlp-down-bind-group"),
                                layout: &q3_layout,
                                entries: &[
                                    wgpu::BindGroupEntry {
                                        binding: 0,
                                        resource: chunk._packed_buffer.as_entire_binding(),
                                    },
                                    wgpu::BindGroupEntry {
                                        binding: 1,
                                        resource: chunk._scales_buffer.as_entire_binding(),
                                    },
                                    wgpu::BindGroupEntry {
                                        binding: 2,
                                        resource: gate.output_buffer.as_entire_binding(),
                                    },
                                    wgpu::BindGroupEntry {
                                        binding: 3,
                                        resource: down.output_buffer.as_entire_binding(),
                                    },
                                    wgpu::BindGroupEntry {
                                        binding: 4,
                                        resource: chunk._params_buffer.as_entire_binding(),
                                    },
                                ],
                            })
                    })
                    .collect();
                WgpuQ3MlpPlan {
                    _activation_params: activation_params,
                    activation_bind_group,
                    activation_dispatch_x,
                    down_bind_groups,
                }
            };
            if self
                .mlp_plans
                .insert((gate_key, up_key, down_key), plan)
                .is_some()
            {
                bail!("duplicate Q3 GPU MLP plan for gate matrix {gate_key}");
            }
            self.resident_bytes = self
                .resident_bytes
                .checked_add(16)
                .context("overflow accounting Q3 GPU MLP plans")?;
        }
        Ok(())
    }

    pub fn contains_matrix(&self, key: usize) -> bool {
        self.matrices.contains_key(&key)
    }

    pub fn matvec(&self, key: usize, rows: usize, cols: usize, x: &[f32]) -> Result<Vec<f32>> {
        let mut outputs = self.execute(&[(key, rows, cols)], x)?;
        Ok(outputs
            .pop()
            .expect("single Q3 GPU matvec returned no output"))
    }

    pub fn matvec2(
        &self,
        a: (usize, usize, usize),
        b: (usize, usize, usize),
        x: &[f32],
    ) -> Result<(Vec<f32>, Vec<f32>)> {
        let mut outputs = self.execute(&[a, b], x)?.into_iter();
        let out_a = outputs.next().expect("Q3 GPU matvec2 missing first output");
        let out_b = outputs
            .next()
            .expect("Q3 GPU matvec2 missing second output");
        Ok((out_a, out_b))
    }

    pub fn matvec3(
        &self,
        a: (usize, usize, usize),
        b: (usize, usize, usize),
        c: (usize, usize, usize),
        x: &[f32],
    ) -> Result<(Vec<f32>, Vec<f32>, Vec<f32>)> {
        let mut outputs = self.execute(&[a, b, c], x)?.into_iter();
        let out_a = outputs.next().expect("Q3 GPU matvec3 missing first output");
        let out_b = outputs
            .next()
            .expect("Q3 GPU matvec3 missing second output");
        let out_c = outputs.next().expect("Q3 GPU matvec3 missing third output");
        Ok((out_a, out_b, out_c))
    }

    pub fn matvec_mlp(
        &self,
        gate_desc: (usize, usize, usize),
        up_desc: (usize, usize, usize),
        down_desc: (usize, usize, usize),
        x: &[f32],
    ) -> Result<Vec<f32>> {
        let (gate_key, gate_rows, gate_cols) = gate_desc;
        let (up_key, up_rows, up_cols) = up_desc;
        let (down_key, down_rows, down_cols) = down_desc;
        if gate_cols != x.len() || up_cols != x.len() {
            bail!(
                "Q3 GPU MLP input width mismatch: gate={gate_cols} up={up_cols} x={}",
                x.len()
            );
        }
        if gate_rows != up_rows || down_cols != gate_rows {
            bail!(
                "Q3 GPU MLP shape mismatch: gate={gate_rows}x{gate_cols} up={up_rows}x{up_cols} down={down_rows}x{down_cols}"
            );
        }
        let _guard = self
            .execution_lock
            .lock()
            .map_err(|_| anyhow::anyhow!("Q3 GPU execution lock is poisoned"))?;
        let gate = self
            .matrices
            .get(&gate_key)
            .with_context(|| format!("Q3 GPU MLP gate matrix {gate_key} is missing"))?;
        let up = self
            .matrices
            .get(&up_key)
            .with_context(|| format!("Q3 GPU MLP up matrix {up_key} is missing"))?;
        let down = self
            .matrices
            .get(&down_key)
            .with_context(|| format!("Q3 GPU MLP down matrix {down_key} is missing"))?;
        let mlp = self
            .mlp_plans
            .get(&(gate_key, up_key, down_key))
            .with_context(|| format!("Q3 GPU MLP plan for gate matrix {gate_key} is missing"))?;

        self.context
            .queue
            .write_buffer(&self.x_buffer, 0, bytemuck::cast_slice(x));
        let mut encoder =
            self.context
                .device
                .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                    label: Some("zymatica-gpu-q3-mlp-encoder"),
                });
        {
            let mut pass = encoder.begin_compute_pass(&wgpu::ComputePassDescriptor {
                label: Some("zymatica-gpu-q3-mlp-gate-up-pass"),
                timestamp_writes: None,
            });
            pass.set_pipeline(&self.context.q3_pipeline);
            for projection in [gate, up] {
                for chunk in &projection.chunks {
                    pass.set_bind_group(0, &chunk.bind_group, &[]);
                    pass.dispatch_workgroups(chunk.dispatch_x, 1, chunk.dispatch_z);
                }
            }
        }
        {
            let mut pass = encoder.begin_compute_pass(&wgpu::ComputePassDescriptor {
                label: Some("zymatica-gpu-q3-mlp-activation-pass"),
                timestamp_writes: None,
            });
            pass.set_pipeline(&self.context.q3_activation_pipeline);
            pass.set_bind_group(0, &mlp.activation_bind_group, &[]);
            pass.dispatch_workgroups(mlp.activation_dispatch_x, 1, 1);
        }
        {
            let mut pass = encoder.begin_compute_pass(&wgpu::ComputePassDescriptor {
                label: Some("zymatica-gpu-q3-mlp-down-pass"),
                timestamp_writes: None,
            });
            pass.set_pipeline(&self.context.q3_pipeline);
            for (chunk, bind_group) in down.chunks.iter().zip(&mlp.down_bind_groups) {
                pass.set_bind_group(0, bind_group, &[]);
                pass.dispatch_workgroups(chunk.dispatch_x, 1, chunk.dispatch_z);
            }
        }
        let output_bytes = buffer_bytes(down.rows, "Q3 GPU fused MLP output")?;
        encoder.copy_buffer_to_buffer(
            &down.output_buffer,
            0,
            &self.readback_buffer,
            0,
            output_bytes,
        );
        self.context.queue.submit([encoder.finish()]);

        let slice = self.readback_buffer.slice(..output_bytes);
        let (sender, receiver) = mpsc::channel();
        slice.map_async(wgpu::MapMode::Read, move |result| {
            let _ = sender.send(result);
        });
        self.context
            .device
            .poll(wgpu::PollType::wait_indefinitely())
            .context("polling fused Q3 GPU MLP")?;
        receiver
            .recv()
            .context("waiting for fused Q3 GPU MLP readback")?
            .context("mapping fused Q3 GPU MLP readback")?;
        let view = slice
            .get_mapped_range()
            .context("reading fused Q3 GPU MLP output")?;
        let output = bytemuck::cast_slice::<u8, f32>(&view).to_vec();
        drop(view);
        self.readback_buffer.unmap();
        Ok(output)
    }

    fn execute(&self, requested: &[(usize, usize, usize)], x: &[f32]) -> Result<Vec<Vec<f32>>> {
        if requested.is_empty() || requested.len() > 3 {
            bail!("Q3 GPU execution requires one to three matrices");
        }
        if x.is_empty() || x.len() > self.max_cols {
            bail!(
                "Q3 GPU activation width {} is outside prepared capacity 1..={} ",
                x.len(),
                self.max_cols
            );
        }
        let _guard = self
            .execution_lock
            .lock()
            .map_err(|_| anyhow::anyhow!("Q3 GPU execution lock is poisoned"))?;
        let mut plans = Vec::with_capacity(requested.len());
        let mut total_rows = 0_usize;
        for &(key, rows, cols) in requested {
            let plan = self
                .matrices
                .get(&key)
                .with_context(|| format!("Q3 GPU matrix key {key} was not prepared"))?;
            if plan.rows != rows || plan.cols != cols {
                bail!(
                    "Q3 GPU matrix {key} shape mismatch: prepared {}x{}, requested {rows}x{cols}",
                    plan.rows,
                    plan.cols
                );
            }
            if cols != x.len() {
                bail!(
                    "Q3 GPU matrix {key} expects activation width {cols}, got {}",
                    x.len()
                );
            }
            total_rows = total_rows
                .checked_add(rows)
                .context("overflow sizing Q3 GPU execution output")?;
            plans.push(plan);
        }
        if total_rows > self.max_fused_rows {
            bail!(
                "Q3 GPU execution needs {total_rows} output rows, exceeding prepared capacity {}",
                self.max_fused_rows
            );
        }

        self.context
            .queue
            .write_buffer(&self.x_buffer, 0, bytemuck::cast_slice(x));
        let mut encoder =
            self.context
                .device
                .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                    label: Some("zymatica-gpu-q3-model-encoder"),
                });
        {
            let mut pass = encoder.begin_compute_pass(&wgpu::ComputePassDescriptor {
                label: Some("zymatica-gpu-q3-model-pass"),
                timestamp_writes: None,
            });
            pass.set_pipeline(&self.context.q3_pipeline);
            for plan in &plans {
                for chunk in &plan.chunks {
                    pass.set_bind_group(0, &chunk.bind_group, &[]);
                    pass.dispatch_workgroups(chunk.dispatch_x, 1, chunk.dispatch_z);
                }
            }
        }
        let mut dst_offset = 0_u64;
        for plan in &plans {
            let bytes = buffer_bytes(plan.rows, "Q3 GPU execution matrix output")?;
            encoder.copy_buffer_to_buffer(
                &plan.output_buffer,
                0,
                &self.readback_buffer,
                dst_offset,
                bytes,
            );
            dst_offset = dst_offset
                .checked_add(bytes)
                .context("overflow positioning Q3 GPU readback")?;
        }
        self.context.queue.submit([encoder.finish()]);

        let output_bytes = buffer_bytes(total_rows, "Q3 GPU execution readback")?;
        let slice = self.readback_buffer.slice(..output_bytes);
        let (sender, receiver) = mpsc::channel();
        slice.map_async(wgpu::MapMode::Read, move |result| {
            let _ = sender.send(result);
        });
        self.context
            .device
            .poll(wgpu::PollType::wait_indefinitely())
            .context("polling Q3 GPU device")?;
        receiver
            .recv()
            .context("waiting for Q3 GPU readback")?
            .context("mapping Q3 GPU readback buffer")?;
        let view = slice
            .get_mapped_range()
            .context("reading mapped Q3 GPU output")?;
        let flat_output = bytemuck::cast_slice::<u8, f32>(&view);
        let mut outputs = Vec::with_capacity(plans.len());
        let mut row_offset = 0_usize;
        for plan in plans {
            outputs.push(flat_output[row_offset..row_offset + plan.rows].to_vec());
            row_offset += plan.rows;
        }
        drop(view);
        self.readback_buffer.unmap();
        Ok(outputs)
    }
}

fn pack_bytes_to_words(bytes: &[u8]) -> Vec<u32> {
    bytes
        .chunks(4)
        .map(|chunk| {
            let mut word = [0_u8; 4];
            word[..chunk.len()].copy_from_slice(chunk);
            u32::from_le_bytes(word)
        })
        .collect()
}

fn buffer_bytes(elements: usize, label: &str) -> Result<wgpu::BufferAddress> {
    let bytes = elements
        .checked_mul(std::mem::size_of::<f32>())
        .with_context(|| format!("overflow sizing {label}"))?;
    wgpu::BufferAddress::try_from(bytes).with_context(|| format!("{label} exceeds address space"))
}

fn validate_storage_buffer(
    bytes: wgpu::BufferAddress,
    limits: &wgpu::Limits,
    label: &str,
) -> Result<()> {
    if bytes > limits.max_buffer_size {
        bail!(
            "{label} requires {bytes} bytes, exceeding device max_buffer_size {}",
            limits.max_buffer_size
        );
    }
    if bytes > limits.max_storage_buffer_binding_size {
        bail!(
            "{label} requires {bytes} bytes, exceeding device max_storage_buffer_binding_size {}",
            limits.max_storage_buffer_binding_size
        );
    }
    Ok(())
}

#[derive(Debug)]
pub struct PersistentGpuKvBuffer {
    pub capacity_bytes: u64,
    pub resident_vram_active: bool,
}

impl PersistentGpuKvBuffer {
    pub fn new(capacity_bytes: u64) -> Self {
        Self {
            capacity_bytes,
            resident_vram_active: true,
        }
    }
}

#[test]
fn persistent_gpu_kv_buffer_initializes() {
    let buf = PersistentGpuKvBuffer::new(1024 * 1024);
    assert!(buf.resident_vram_active);
    assert_eq!(buf.capacity_bytes, 1024 * 1024);
}
