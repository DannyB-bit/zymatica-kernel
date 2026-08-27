use crate::ops::{
    apply_rope_split_half_cached, gelu_pytorch_tanh, matvec, matvec2, matvec3, rms_norm,
    rms_norm_chunks_in_place, rms_norm_in_place, rms_norm_unit_chunks_in_place, softcap_in_place,
    softmax_in_place,
};
use crate::quant::{QuantMatrix, QuantizedActivationMode, RowQ4Matrix, RowQ5Matrix, RowQ8Matrix};
use crate::sampling::{SamplingConfig, sample_next};
use crate::tensor::{Matrix, Tensor3};
use crate::weights::LazyRowTensor;
use anyhow::{Context, Result, bail};
use rand::{Rng, SeedableRng, rngs::StdRng};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::{Path, PathBuf};
#[cfg(feature = "gpu")]
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex, OnceLock};

const PER_LAYER_INPUT_CACHE_LIMIT: usize = 64;
pub type PerLayerInputs = Arc<Vec<Vec<f32>>>;
pub type PerLayerInputCache = Arc<Mutex<HashMap<usize, PerLayerInputs>>>;

fn new_per_layer_input_cache() -> PerLayerInputCache {
    Arc::new(Mutex::new(HashMap::new()))
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StaticInstruction {
    LayerStart { layer_idx: usize },
    InputNorm { layer_idx: usize },
    ProjQkv { layer_idx: usize },
    RopeAndCache { layer_idx: usize },
    Attention { layer_idx: usize },
    ProjO { layer_idx: usize },
    AddResidualAttn { layer_idx: usize },
    FeedForwardNorm { layer_idx: usize },
    ProjMlpGateUp { layer_idx: usize },
    ProjMlpDown { layer_idx: usize },
    AddResidualMlp { layer_idx: usize },
    CheckEarlyExit { layer_idx: usize },
}

pub fn compile_instructions(num_layers: usize) -> Vec<StaticInstruction> {
    let mut instructions = Vec::with_capacity(num_layers * 12);
    for layer_idx in 0..num_layers {
        instructions.push(StaticInstruction::LayerStart { layer_idx });
        instructions.push(StaticInstruction::InputNorm { layer_idx });
        instructions.push(StaticInstruction::ProjQkv { layer_idx });
        instructions.push(StaticInstruction::RopeAndCache { layer_idx });
        instructions.push(StaticInstruction::Attention { layer_idx });
        instructions.push(StaticInstruction::ProjO { layer_idx });
        instructions.push(StaticInstruction::AddResidualAttn { layer_idx });
        instructions.push(StaticInstruction::FeedForwardNorm { layer_idx });
        instructions.push(StaticInstruction::ProjMlpGateUp { layer_idx });
        instructions.push(StaticInstruction::ProjMlpDown { layer_idx });
        instructions.push(StaticInstruction::AddResidualMlp { layer_idx });
        instructions.push(StaticInstruction::CheckEarlyExit { layer_idx });
    }
    instructions
}

fn hidden_state_variance(values: &[f32]) -> f32 {
    if values.is_empty() {
        return 0.0;
    }
    let mean = values.iter().copied().sum::<f32>() / values.len() as f32;
    values
        .iter()
        .map(|value| {
            let diff = *value - mean;
            diff * diff
        })
        .sum::<f32>()
        / values.len() as f32
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GemmaConfig {
    pub vocab_size: usize,
    pub hidden_size: usize,
    pub intermediate_size: usize,
    pub num_hidden_layers: usize,
    pub num_attention_heads: usize,
    pub num_key_value_heads: usize,
    pub num_kv_shared_layers: usize,
    pub head_dim: usize,
    pub global_head_dim: Option<usize>,
    pub rms_norm_eps: f32,
    pub rope_theta: f32,
    pub full_attention_rope_theta: Option<f32>,
    pub full_attention_rotary_fraction: f32,
    pub max_position_embeddings: usize,
    pub sliding_window: Option<usize>,
    pub layer_types: Vec<String>,
    pub hidden_size_per_layer_input: Option<usize>,
    pub vocab_size_per_layer_input: Option<usize>,
    pub hidden_activation: String,
    pub embedding_scale: f32,
    pub per_layer_embedding_scale: f32,
    pub final_logit_softcapping: Option<f32>,
    #[serde(default)]
    pub fold_rms_norm: bool,
}

impl GemmaConfig {
    pub fn tiny_for_tests() -> Self {
        Self {
            vocab_size: 32,
            hidden_size: 16,
            intermediate_size: 32,
            num_hidden_layers: 2,
            num_attention_heads: 4,
            num_key_value_heads: 2,
            num_kv_shared_layers: 0,
            head_dim: 4,
            global_head_dim: None,
            rms_norm_eps: 1e-6,
            rope_theta: 10_000.0,
            full_attention_rope_theta: None,
            full_attention_rotary_fraction: 1.0,
            max_position_embeddings: 512,
            sliding_window: None,
            layer_types: vec!["sliding_attention".to_string(); 2],
            hidden_size_per_layer_input: None,
            vocab_size_per_layer_input: None,
            hidden_activation: "silu".to_string(),
            embedding_scale: 1.0,
            per_layer_embedding_scale: 1.0,
            final_logit_softcapping: None,
            fold_rms_norm: false,
        }
    }

    pub fn e4b_mock_config() -> Self {
        Self {
            vocab_size: 32,
            hidden_size: 96,
            intermediate_size: 128,
            num_hidden_layers: 2,
            num_attention_heads: 12,
            num_key_value_heads: 4,
            num_kv_shared_layers: 0,
            head_dim: 8,
            global_head_dim: None,
            rms_norm_eps: 1e-6,
            rope_theta: 10_000.0,
            full_attention_rope_theta: None,
            full_attention_rotary_fraction: 1.0,
            max_position_embeddings: 512,
            sliding_window: None,
            layer_types: vec!["full_attention".to_string(); 2],
            hidden_size_per_layer_input: None,
            vocab_size_per_layer_input: None,
            hidden_activation: "gelu_pytorch_tanh".to_string(),
            embedding_scale: 1.0,
            per_layer_embedding_scale: 1.0,
            final_logit_softcapping: None,
            fold_rms_norm: false,
        }
    }

    pub fn validate(&self) {
        assert_eq!(self.layer_types.len(), self.num_hidden_layers);
        assert_eq!(self.num_attention_heads % self.num_key_value_heads, 0);
    }

    pub fn layer_rope_theta(&self, layer_idx: usize) -> f32 {
        if self.layer_types[layer_idx] == "full_attention" {
            self.full_attention_rope_theta.unwrap_or(self.rope_theta)
        } else {
            self.rope_theta
        }
    }

    pub fn layer_rotary_fraction(&self, layer_idx: usize) -> f32 {
        if self.layer_types[layer_idx] == "full_attention" {
            self.full_attention_rotary_fraction
        } else {
            1.0
        }
    }

    pub fn layer_sliding_window(&self, layer_idx: usize) -> Option<usize> {
        if self.layer_types[layer_idx] == "sliding_attention" {
            self.sliding_window
        } else {
            None
        }
    }

    pub fn shared_kv_source_layer(&self, layer_idx: usize) -> Option<usize> {
        if self.num_kv_shared_layers == 0 || self.num_kv_shared_layers >= self.num_hidden_layers {
            return None;
        }
        let first_shared = self.num_hidden_layers - self.num_kv_shared_layers;
        if layer_idx < first_shared {
            return None;
        }
        let layer_type = &self.layer_types[layer_idx];
        (0..first_shared)
            .rev()
            .find(|idx| &self.layer_types[*idx] == layer_type)
    }
}

#[derive(Debug, Clone)]
pub struct LayerWeights {
    pub input_norm: Vec<f32>,
    pub post_attention_norm: Vec<f32>,
    pub pre_feedforward_norm: Vec<f32>,
    pub post_feedforward_norm: Vec<f32>,
    pub q_norm: Vec<f32>,
    pub k_norm: Vec<f32>,
    pub q_proj: Matrix,
    pub k_proj: Matrix,
    pub v_proj: Matrix,
    pub o_proj: Matrix,
    pub gate_proj: Matrix,
    pub up_proj: Matrix,
    pub down_proj: Matrix,
    pub layer_scalar: f32,
    pub per_layer_input_gate: Option<Matrix>,
    pub per_layer_projection: Option<Matrix>,
    pub post_per_layer_input_norm: Option<Vec<f32>>,
}

#[derive(Debug, Clone)]
pub struct LoraProjection {
    pub a: Matrix,
    pub b: Matrix,
    pub alpha: f32,
}

#[derive(Debug, Clone, Default)]
pub struct LayerLoraAdapters {
    pub q_proj: Option<LoraProjection>,
    pub k_proj: Option<LoraProjection>,
    pub v_proj: Option<LoraProjection>,
    pub o_proj: Option<LoraProjection>,
}

#[derive(Debug, Clone, Default)]
pub struct LoraAdapters {
    pub layers: Vec<LayerLoraAdapters>,
}

#[derive(Debug, Clone, Copy, Deserialize)]
struct LoraAdapterConfig {
    r: Option<usize>,
    lora_alpha: Option<f32>,
}

#[derive(Debug, Clone, Copy)]
struct LayerLoraDims {
    q_input: usize,
    q_output: usize,
    k_input: usize,
    k_output: usize,
    v_input: usize,
    v_output: usize,
    o_input: usize,
    o_output: usize,
}

impl LoraProjection {
    pub fn validate(&self, input_dim: usize, output_dim: usize, role: &str) {
        assert_eq!(self.a.cols, input_dim, "LoRA {role} A input dimension");
        assert_eq!(self.b.rows, output_dim, "LoRA {role} B output dimension");
        assert_eq!(self.a.rows, self.b.cols, "LoRA {role} rank dimension");
    }

    pub fn apply_to(&self, output: &mut [f32], input: &[f32]) {
        self.validate(input.len(), output.len(), "projection");
        let rank = self.a.rows.max(1);
        let hidden = matvec(&self.a, input);
        let delta = matvec(&self.b, &hidden);
        let scale = self.alpha / rank as f32;
        for (dst, value) in output.iter_mut().zip(delta) {
            *dst += value * scale;
        }
    }
}

fn validate_lora_adapters(cfg: &GemmaConfig, layers: &[LayerWeights], lora: &LoraAdapters) {
    assert!(
        lora.layers.len() <= layers.len(),
        "LoRA adapter has more layers than the base model"
    );
    for (idx, layer_lora) in lora.layers.iter().enumerate() {
        let layer = &layers[idx];
        if let Some(proj) = &layer_lora.q_proj {
            proj.validate(cfg.hidden_size, layer.q_proj.rows, "q_proj");
        }
        if let Some(proj) = &layer_lora.k_proj {
            proj.validate(cfg.hidden_size, layer.k_proj.rows, "k_proj");
        }
        if let Some(proj) = &layer_lora.v_proj {
            proj.validate(cfg.hidden_size, layer.v_proj.rows, "v_proj");
        }
        if let Some(proj) = &layer_lora.o_proj {
            proj.validate(layer.o_proj.cols, layer.o_proj.rows, "o_proj");
        }
    }
}

fn validate_quantized_lora_adapters(
    cfg: &GemmaConfig,
    layers: &[QuantizedLayer],
    lora: &LoraAdapters,
) {
    assert!(
        lora.layers.len() <= layers.len(),
        "LoRA adapter has more layers than the base model"
    );
    for (idx, layer_lora) in lora.layers.iter().enumerate() {
        let layer = &layers[idx];
        if let Some(proj) = &layer_lora.q_proj {
            proj.validate(cfg.hidden_size, layer.q_proj.rows(), "q_proj");
        }
        if let Some(proj) = &layer_lora.k_proj {
            proj.validate(cfg.hidden_size, layer.k_proj.rows(), "k_proj");
        }
        if let Some(proj) = &layer_lora.v_proj {
            proj.validate(cfg.hidden_size, layer.v_proj.rows(), "v_proj");
        }
        if let Some(proj) = &layer_lora.o_proj {
            proj.validate(layer.o_proj.cols(), layer.o_proj.rows(), "o_proj");
        }
    }
}

fn layer_lora(lora: Option<&LoraAdapters>, layer_idx: usize) -> Option<&LayerLoraAdapters> {
    lora.and_then(|adapters| adapters.layers.get(layer_idx))
}

fn semantic_early_exit_threshold() -> Option<f32> {
    static THRESHOLD: OnceLock<Option<f32>> = OnceLock::new();
    *THRESHOLD.get_or_init(|| {
        std::env::var("ZYMATICA_EARLY_EXIT_THRESHOLD")
            .ok()
            .and_then(|threshold| threshold.parse::<f32>().ok())
    })
}

fn attention_sparsity_enabled() -> bool {
    static ENABLED: OnceLock<bool> = OnceLock::new();
    *ENABLED.get_or_init(|| std::env::var("ZYMATICA_ATTENTION_SPARSITY").is_ok())
}

fn should_semantic_early_exit(layer_idx: usize, values: &[f32], threshold: f32) -> bool {
    if layer_idx != 6 || values.is_empty() {
        return false;
    }
    hidden_state_variance(values) < threshold
}

fn apply_lora_if_present(output: &mut [f32], input: &[f32], projection: Option<&LoraProjection>) {
    if let Some(projection) = projection {
        projection.apply_to(output, input);
    }
}

fn load_lora_adapters_from_dir(
    adapter_dir: impl AsRef<Path>,
    layer_dims: &[LayerLoraDims],
) -> Result<LoraAdapters> {
    let adapter_dir = adapter_dir.as_ref();
    let config = load_lora_adapter_config(adapter_dir)?;
    let mut reader = crate::weights::TensorReader::from_dir(adapter_dir)
        .with_context(|| format!("indexing LoRA adapter tensors in {}", adapter_dir.display()))?;
    let names: Vec<String> = reader.index().names().map(ToOwned::to_owned).collect();
    let mut layers = vec![LayerLoraAdapters::default(); layer_dims.len()];
    let mut loaded = 0_usize;

    for (layer_idx, dims) in layer_dims.iter().copied().enumerate() {
        layers[layer_idx].q_proj = load_optional_lora_projection(
            &mut reader,
            &names,
            layer_idx,
            "q_proj",
            dims.q_input,
            dims.q_output,
            config,
        )?;
        layers[layer_idx].k_proj = load_optional_lora_projection(
            &mut reader,
            &names,
            layer_idx,
            "k_proj",
            dims.k_input,
            dims.k_output,
            config,
        )?;
        layers[layer_idx].v_proj = load_optional_lora_projection(
            &mut reader,
            &names,
            layer_idx,
            "v_proj",
            dims.v_input,
            dims.v_output,
            config,
        )?;
        layers[layer_idx].o_proj = load_optional_lora_projection(
            &mut reader,
            &names,
            layer_idx,
            "o_proj",
            dims.o_input,
            dims.o_output,
            config,
        )?;
        loaded += layers[layer_idx].q_proj.is_some() as usize
            + layers[layer_idx].k_proj.is_some() as usize
            + layers[layer_idx].v_proj.is_some() as usize
            + layers[layer_idx].o_proj.is_some() as usize;
    }

    if loaded == 0 {
        bail!(
            "LoRA adapter {} did not contain q/k/v/o projection adapter tensors",
            adapter_dir.display()
        );
    }
    Ok(LoraAdapters { layers })
}

fn load_lora_adapter_config(adapter_dir: &Path) -> Result<LoraAdapterConfig> {
    let config_path = adapter_dir.join("adapter_config.json");
    if !config_path.exists() {
        return Ok(LoraAdapterConfig {
            r: None,
            lora_alpha: None,
        });
    }
    let bytes = std::fs::read(&config_path)
        .with_context(|| format!("reading LoRA adapter config {}", config_path.display()))?;
    serde_json::from_slice(&bytes)
        .with_context(|| format!("parsing LoRA adapter config {}", config_path.display()))
}

fn load_optional_lora_projection(
    reader: &mut crate::weights::TensorReader,
    names: &[String],
    layer_idx: usize,
    projection: &str,
    input_dim: usize,
    output_dim: usize,
    config: LoraAdapterConfig,
) -> Result<Option<LoraProjection>> {
    let a_name = find_lora_tensor_name(names, layer_idx, projection, "lora_A");
    let b_name = find_lora_tensor_name(names, layer_idx, projection, "lora_B");
    match (a_name, b_name) {
        (None, None) => Ok(None),
        (Some(a), Some(b)) => Ok(Some(load_lora_projection(
            reader, &a, &b, input_dim, output_dim, config,
        )?)),
        (Some(a), None) => {
            bail!("LoRA projection {projection} layer {layer_idx} has A tensor {a} but no B tensor")
        }
        (None, Some(b)) => {
            bail!("LoRA projection {projection} layer {layer_idx} has B tensor {b} but no A tensor")
        }
    }
}

fn find_lora_tensor_name(
    names: &[String],
    layer_idx: usize,
    projection: &str,
    adapter_half: &str,
) -> Option<String> {
    let layer_marker = format!("layers.{layer_idx}.");
    let projection_marker = format!(".{projection}.");
    let suffix_plain = format!(".{adapter_half}.weight");
    let suffix_default = format!(".{adapter_half}.default.weight");
    let mut matches: Vec<_> = names
        .iter()
        .filter(|name| {
            name.contains(&layer_marker)
                && name.contains(&projection_marker)
                && (name.ends_with(&suffix_plain) || name.ends_with(&suffix_default))
        })
        .cloned()
        .collect();
    matches.sort();
    matches.into_iter().next()
}

fn load_lora_projection(
    reader: &mut crate::weights::TensorReader,
    a_name: &str,
    b_name: &str,
    input_dim: usize,
    output_dim: usize,
    config: LoraAdapterConfig,
) -> Result<LoraProjection> {
    let (a_shape, a_data) = reader
        .read_f32(a_name)
        .with_context(|| format!("loading LoRA A tensor {a_name}"))?;
    let (b_shape, b_data) = reader
        .read_f32(b_name)
        .with_context(|| format!("loading LoRA B tensor {b_name}"))?;
    if a_shape.len() != 2 || a_shape[1] != input_dim {
        bail!(
            "LoRA A tensor {a_name} shape mismatch: expected [rank, {input_dim}], got {:?}",
            a_shape
        );
    }
    let rank = a_shape[0];
    if let Some(expected_rank) = config.r
        && expected_rank != rank
    {
        bail!("LoRA rank mismatch for {a_name}: config r={expected_rank}, tensor rank={rank}");
    }
    if b_shape != [output_dim, rank] {
        bail!(
            "LoRA B tensor {b_name} shape mismatch: expected [{output_dim}, {rank}], got {:?}",
            b_shape
        );
    }
    Ok(LoraProjection {
        a: Matrix::from_row_major(rank, input_dim, a_data),
        b: Matrix::from_row_major(output_dim, rank, b_data),
        alpha: config.lora_alpha.unwrap_or(rank as f32),
    })
}

impl LayerWeights {
    pub fn head_dim(&self, cfg: &GemmaConfig) -> usize {
        debug_assert_eq!(self.q_proj.rows % cfg.num_attention_heads, 0);
        self.q_proj.rows / cfg.num_attention_heads
    }

    pub fn kv_heads(&self, cfg: &GemmaConfig) -> usize {
        let head_dim = self.head_dim(cfg);
        debug_assert_eq!(self.k_proj.rows % head_dim, 0);
        self.k_proj.rows / head_dim
    }
}

#[derive(Debug, Clone)]
pub struct KVCache {
    pub keys: Vec<Tensor3>,
    pub values: Vec<Tensor3>,
    pub max_seq: usize,
}

#[derive(Debug, Clone, Copy)]
pub struct SharedPagedKvCache(pub *mut crate::paged_kv::PagedKvCache);

unsafe impl Send for SharedPagedKvCache {}
unsafe impl Sync for SharedPagedKvCache {}

#[derive(Debug, Clone)]
pub enum AnyKvCache {
    Dense(KVCache),
    Paged {
        cache: SharedPagedKvCache,
        sequence_id: u64,
    },
}

impl AnyKvCache {
    pub fn new_dense(layers: usize, max_seq: usize, kv_heads: usize, head_dim: usize) -> Self {
        Self::Dense(KVCache::new(layers, max_seq, kv_heads, head_dim))
    }

    pub fn set_kv(
        &mut self,
        layer: usize,
        position: usize,
        head: usize,
        key: &[f32],
        value: &[f32],
    ) {
        match self {
            Self::Dense(cache) => {
                cache.keys[layer]
                    .get_mut(position, head)
                    .copy_from_slice(key);
                cache.values[layer]
                    .get_mut(position, head)
                    .copy_from_slice(value);
            }
            Self::Paged { cache, sequence_id } => unsafe {
                (*cache.0).set_kv(*sequence_id, position, layer, head, key, value);
            },
        }
    }

    pub fn get_key(&self, layer: usize, position: usize, kv_head: usize) -> &[f32] {
        match self {
            Self::Dense(cache) => cache.keys[layer].get(position, kv_head),
            Self::Paged { cache, sequence_id } => unsafe {
                (*cache.0).key(*sequence_id, position, layer, kv_head)
            },
        }
    }

    pub fn get_value(&self, layer: usize, position: usize, kv_head: usize) -> &[f32] {
        match self {
            Self::Dense(cache) => cache.values[layer].get(position, kv_head),
            Self::Paged { cache, sequence_id } => unsafe {
                (*cache.0).value(*sequence_id, position, layer, kv_head)
            },
        }
    }
}

#[derive(Debug, Clone, Copy)]
struct AttentionPlan {
    kv_layer_idx: usize,
    query_layer_idx: usize,
    head_dim: usize,
    kv_heads: usize,
}

impl KVCache {
    pub fn new(layers: usize, max_seq: usize, kv_heads: usize, head_dim: usize) -> Self {
        let keys = (0..layers)
            .map(|_| Tensor3::zeros(max_seq, kv_heads, head_dim))
            .collect();
        let values = (0..layers)
            .map(|_| Tensor3::zeros(max_seq, kv_heads, head_dim))
            .collect();
        Self {
            keys,
            values,
            max_seq,
        }
    }

    pub fn new_with_layer_shapes(max_seq: usize, layer_shapes: &[(usize, usize)]) -> Self {
        let keys = layer_shapes
            .iter()
            .map(|(kv_heads, head_dim)| Tensor3::zeros(max_seq, *kv_heads, *head_dim))
            .collect();
        let values = layer_shapes
            .iter()
            .map(|(kv_heads, head_dim)| Tensor3::zeros(max_seq, *kv_heads, *head_dim))
            .collect();
        Self {
            keys,
            values,
            max_seq,
        }
    }
}

#[derive(Debug, Clone)]
pub struct ForwardOutput {
    pub logits: Vec<f32>,
    pub hidden_state: Vec<f32>,
}

#[derive(Debug, Clone)]
pub struct NativeGemma {
    pub cfg: GemmaConfig,
    pub token_embedding: Matrix,
    pub token_embedding_per_layer: Option<RowMatrix>,
    pub per_layer_model_projection: Option<Matrix>,
    pub per_layer_projection_norm: Option<Vec<f32>>,
    pub per_layer_input_cache: PerLayerInputCache,
    pub layers: Vec<LayerWeights>,
    pub lora: Option<LoraAdapters>,
    pub final_norm: Vec<f32>,
    pub lm_head: Matrix,
    pub rope_tables: Vec<crate::ops::RopeTrigTable>,
    pub instructions: Vec<StaticInstruction>,
}

#[derive(Debug, Clone)]
pub enum RowMatrix {
    Dense(Matrix),
    Lazy(LazyRowTensor),
}

impl RowMatrix {
    pub fn row(&self, row: usize) -> Vec<f32> {
        match self {
            Self::Dense(matrix) => matrix.row(row).to_vec(),
            Self::Lazy(tensor) => tensor
                .row_f32(row)
                .unwrap_or_else(|err| panic!("lazy row tensor read failed: {err:#}")),
        }
    }

    pub fn rows(&self) -> usize {
        match self {
            Self::Dense(matrix) => matrix.rows,
            Self::Lazy(tensor) => tensor.rows(),
        }
    }

    pub fn cols(&self) -> usize {
        match self {
            Self::Dense(matrix) => matrix.cols,
            Self::Lazy(tensor) => tensor.cols(),
        }
    }

    pub fn as_dense(&self) -> Option<&Matrix> {
        match self {
            Self::Dense(matrix) => Some(matrix),
            Self::Lazy(_) => None,
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub enum ModelSource<'a> {
    Dir(&'a Path),
    InMemory {
        config_json: &'a [u8],
        files: &'a HashMap<String, Arc<Vec<u8>>>,
    },
}

impl NativeGemma {
    pub fn from_hf_dir(model_dir: impl AsRef<Path>) -> Result<Self> {
        Self::from_source(ModelSource::Dir(model_dir.as_ref()))
    }

    pub fn from_source(source: ModelSource<'_>) -> Result<Self> {
        let resolution = match source {
            ModelSource::Dir(model_dir) => crate::gemma_hf::resolve_gemma_dir(model_dir)
                .with_context(|| format!("resolving Gemma model at {}", model_dir.display()))?,
            ModelSource::InMemory { config_json, files } => {
                let index = crate::weights::TensorIndex::from_in_memory(files)?;
                crate::gemma_hf::resolve_gemma_in_memory(config_json, &index)?
            }
        };
        let cfg = resolution.config;
        cfg.validate();

        let mut reader = match source {
            ModelSource::Dir(model_dir) => crate::weights::TensorReader::from_dir(model_dir)
                .with_context(|| format!("indexing safetensors in {}", model_dir.display()))?,
            ModelSource::InMemory { files, .. } => {
                crate::weights::TensorReader::from_in_memory(files.clone())?
            }
        };

        let roles: HashMap<_, _> = resolution
            .tensors
            .iter()
            .filter_map(|tensor| {
                tensor
                    .name
                    .as_ref()
                    .map(|name| (tensor.role.as_str(), name.as_str()))
            })
            .collect();

        let token_embedding_name = required_role(&roles, "token_embedding")?;
        let token_embedding = load_matrix(
            &mut reader,
            token_embedding_name,
            cfg.vocab_size,
            cfg.hidden_size,
            "token_embedding",
        )?;
        let token_embedding_per_layer = if let Some(name) =
            roles.get("token_embedding_per_layer").copied()
        {
            let per_layer_dim = cfg.hidden_size_per_layer_input.with_context(|| {
                "token_embedding_per_layer tensor found but config lacks hidden_size_per_layer_input"
            })?;
            let table = LazyRowTensor::from_reader(&mut reader, name).with_context(|| {
                format!("mapping tensor role=token_embedding_per_layer name={name}")
            })?;
            let expected_rows = cfg.vocab_size_per_layer_input.unwrap_or(cfg.vocab_size);
            let expected_cols = cfg.num_hidden_layers * per_layer_dim;
            if table.rows() != expected_rows || table.cols() != expected_cols {
                bail!(
                    "shape mismatch for role=token_embedding_per_layer name={name}: expected [{expected_rows}, {expected_cols}] got [{}, {}]",
                    table.rows(),
                    table.cols()
                );
            }
            Some(RowMatrix::Lazy(table))
        } else {
            None
        };
        let per_layer_model_projection = if let Some(name) =
            roles.get("per_layer_model_projection").copied()
        {
            let per_layer_dim = cfg.hidden_size_per_layer_input.with_context(|| {
                    "per_layer_model_projection tensor found but config lacks hidden_size_per_layer_input"
                })?;
            Some(load_matrix(
                &mut reader,
                name,
                cfg.num_hidden_layers * per_layer_dim,
                cfg.hidden_size,
                "per_layer_model_projection",
            )?)
        } else {
            None
        };
        let per_layer_projection_norm = if let Some(name) =
            roles.get("per_layer_projection_norm").copied()
        {
            Some(load_norm(
                    &mut reader,
                    name,
                    cfg.hidden_size_per_layer_input.with_context(|| {
                        "per_layer_projection_norm tensor found but config lacks hidden_size_per_layer_input"
                    })?,
                    "per_layer_projection_norm",
                    cfg.fold_rms_norm,
                )?)
        } else {
            None
        };
        let final_norm = load_norm(
            &mut reader,
            required_role(&roles, "final_norm")?,
            cfg.hidden_size,
            "final_norm",
            cfg.fold_rms_norm,
        )?;

        let lm_head = if let Some(name) = roles.get("lm_head").copied() {
            load_matrix(
                &mut reader,
                name,
                cfg.vocab_size,
                cfg.hidden_size,
                "lm_head",
            )?
        } else {
            token_embedding.clone()
        };

        let mut layers = Vec::with_capacity(cfg.num_hidden_layers);
        for layer in 0..cfg.num_hidden_layers {
            let role = |suffix: &str| format!("layers.{layer}.{suffix}");
            let input_norm = load_norm(
                &mut reader,
                required_role(&roles, &role("input_norm"))?,
                cfg.hidden_size,
                &role("input_norm"),
                cfg.fold_rms_norm,
            )?;
            let post_attention_norm = load_norm(
                &mut reader,
                required_role(&roles, &role("post_attention_norm"))?,
                cfg.hidden_size,
                &role("post_attention_norm"),
                cfg.fold_rms_norm,
            )?;
            let pre_feedforward_norm = load_optional_norm(
                &mut reader,
                roles.get(role("pre_feedforward_norm").as_str()).copied(),
                cfg.hidden_size,
                &role("pre_feedforward_norm"),
                cfg.fold_rms_norm,
            )?
            .unwrap_or_else(|| post_attention_norm.clone());
            let post_feedforward_norm = load_optional_norm(
                &mut reader,
                roles.get(role("post_feedforward_norm").as_str()).copied(),
                cfg.hidden_size,
                &role("post_feedforward_norm"),
                cfg.fold_rms_norm,
            )?
            .unwrap_or_else(|| vec![1.0; cfg.hidden_size]);
            let q_proj = load_matrix_with_cols(
                &mut reader,
                required_role(&roles, &role("q_proj"))?,
                cfg.hidden_size,
                &role("q_proj"),
            )?;
            if q_proj.rows % cfg.num_attention_heads != 0 {
                bail!(
                    "shape mismatch for role={}: q rows {} are not divisible by attention heads {}",
                    role("q_proj"),
                    q_proj.rows,
                    cfg.num_attention_heads
                );
            }
            let layer_head_dim = q_proj.rows / cfg.num_attention_heads;
            let k_proj = load_matrix_with_cols(
                &mut reader,
                required_role(&roles, &role("k_proj"))?,
                cfg.hidden_size,
                &role("k_proj"),
            )?;
            let v_proj = load_matrix_with_cols(
                &mut reader,
                required_role(&roles, &role("v_proj"))?,
                cfg.hidden_size,
                &role("v_proj"),
            )?;
            if k_proj.rows != v_proj.rows || k_proj.rows % layer_head_dim != 0 {
                bail!(
                    "shape mismatch for layer {layer}: k/v rows {} / {} not compatible with head_dim {}",
                    k_proj.rows,
                    v_proj.rows,
                    layer_head_dim
                );
            }
            let o_proj = load_matrix(
                &mut reader,
                required_role(&roles, &role("o_proj"))?,
                cfg.hidden_size,
                q_proj.rows,
                &role("o_proj"),
            )?;
            let q_norm = load_optional_norm(
                &mut reader,
                roles.get(role("q_norm").as_str()).copied(),
                layer_head_dim,
                &role("q_norm"),
                cfg.fold_rms_norm,
            )?
            .unwrap_or_else(|| vec![1.0; layer_head_dim]);
            let k_norm = load_optional_norm(
                &mut reader,
                roles.get(role("k_norm").as_str()).copied(),
                layer_head_dim,
                &role("k_norm"),
                cfg.fold_rms_norm,
            )?
            .unwrap_or_else(|| vec![1.0; layer_head_dim]);
            let gate_proj = load_matrix_with_cols(
                &mut reader,
                required_role(&roles, &role("gate_proj"))?,
                cfg.hidden_size,
                &role("gate_proj"),
            )?;
            let up_proj = load_matrix_with_cols(
                &mut reader,
                required_role(&roles, &role("up_proj"))?,
                cfg.hidden_size,
                &role("up_proj"),
            )?;
            if up_proj.rows != gate_proj.rows {
                bail!(
                    "shape mismatch for layer {layer}: gate rows {} != up rows {}",
                    gate_proj.rows,
                    up_proj.rows
                );
            }
            let down_proj = load_matrix(
                &mut reader,
                required_role(&roles, &role("down_proj"))?,
                cfg.hidden_size,
                gate_proj.rows,
                &role("down_proj"),
            )?;
            let layer_scalar = load_optional_vec(
                &mut reader,
                roles.get(role("layer_scalar").as_str()).copied(),
                1,
                &role("layer_scalar"),
            )?
            .map(|v| v[0])
            .unwrap_or(1.0);
            let per_layer_input_gate =
                if let Some(name) = roles.get(role("per_layer_input_gate").as_str()).copied() {
                    Some(load_matrix_with_cols(
                        &mut reader,
                        name,
                        cfg.hidden_size,
                        &role("per_layer_input_gate"),
                    )?)
                } else {
                    None
                };
            let per_layer_projection =
                if let Some(name) = roles.get(role("per_layer_projection").as_str()).copied() {
                    let input_dim = per_layer_input_gate
                    .as_ref()
                    .map(|m| m.rows)
                    .or(cfg.hidden_size_per_layer_input)
                    .with_context(|| {
                        format!(
                            "{} present but no per-layer input gate or hidden_size_per_layer_input",
                            role("per_layer_projection")
                        )
                    })?;
                    Some(load_matrix(
                        &mut reader,
                        name,
                        cfg.hidden_size,
                        input_dim,
                        &role("per_layer_projection"),
                    )?)
                } else {
                    None
                };
            let post_per_layer_input_norm = load_optional_norm(
                &mut reader,
                roles
                    .get(role("post_per_layer_input_norm").as_str())
                    .copied(),
                cfg.hidden_size,
                &role("post_per_layer_input_norm"),
                cfg.fold_rms_norm,
            )?;

            layers.push(LayerWeights {
                input_norm,
                post_attention_norm,
                pre_feedforward_norm,
                post_feedforward_norm,
                q_norm,
                k_norm,
                q_proj,
                k_proj,
                v_proj,
                o_proj,
                gate_proj,
                up_proj,
                down_proj,
                layer_scalar,
                per_layer_input_gate,
                per_layer_projection,
                post_per_layer_input_norm,
            });
        }

        let mut rope_tables = Vec::with_capacity(layers.len());
        for (layer_idx, layer) in layers.iter().enumerate() {
            let head_dim = layer.head_dim(&cfg);
            let rotary_fraction = cfg.layer_rotary_fraction(layer_idx);
            let rope_theta = cfg.layer_rope_theta(layer_idx);
            let max_pos = cfg.max_position_embeddings;
            rope_tables.push(crate::ops::RopeTrigTable::new(
                max_pos,
                head_dim,
                rotary_fraction,
                rope_theta,
            ));
        }

        let instructions = compile_instructions(layers.len());

        Ok(Self {
            cfg,
            token_embedding,
            token_embedding_per_layer,
            per_layer_model_projection,
            per_layer_projection_norm,
            per_layer_input_cache: new_per_layer_input_cache(),
            layers,
            lora: None,
            final_norm,
            lm_head,
            rope_tables,
            instructions,
        })
    }

    pub fn seeded_tiny(seed: u64) -> Self {
        Self::seeded_model(GemmaConfig::tiny_for_tests(), seed)
    }

    pub fn seeded_e4b_mock(seed: u64) -> Self {
        Self::seeded_model(GemmaConfig::e4b_mock_config(), seed)
    }

    pub fn seeded_model(cfg: GemmaConfig, seed: u64) -> Self {
        cfg.validate();
        let mut rng = StdRng::seed_from_u64(seed);
        fn rand_matrix(rng: &mut StdRng, rows: usize, cols: usize) -> Matrix {
            let scale = (2.0 / (rows + cols) as f32).sqrt();
            Matrix::from_row_major(
                rows,
                cols,
                (0..rows * cols)
                    .map(|_| rng.gen_range(-scale..scale))
                    .collect(),
            )
        }

        fn rand_norm(rng: &mut StdRng, n: usize) -> Vec<f32> {
            (0..n).map(|_| rng.gen_range(0.8..1.2)).collect()
        }

        let layers: Vec<LayerWeights> = (0..cfg.num_hidden_layers)
            .map(|_| LayerWeights {
                input_norm: rand_norm(&mut rng, cfg.hidden_size),
                post_attention_norm: rand_norm(&mut rng, cfg.hidden_size),
                pre_feedforward_norm: rand_norm(&mut rng, cfg.hidden_size),
                post_feedforward_norm: rand_norm(&mut rng, cfg.hidden_size),
                q_norm: rand_norm(&mut rng, cfg.head_dim),
                k_norm: rand_norm(&mut rng, cfg.head_dim),
                q_proj: rand_matrix(
                    &mut rng,
                    cfg.num_attention_heads * cfg.head_dim,
                    cfg.hidden_size,
                ),
                k_proj: rand_matrix(
                    &mut rng,
                    cfg.num_key_value_heads * cfg.head_dim,
                    cfg.hidden_size,
                ),
                v_proj: rand_matrix(
                    &mut rng,
                    cfg.num_key_value_heads * cfg.head_dim,
                    cfg.hidden_size,
                ),
                o_proj: rand_matrix(&mut rng, cfg.hidden_size, cfg.hidden_size),
                gate_proj: rand_matrix(&mut rng, cfg.intermediate_size, cfg.hidden_size),
                up_proj: rand_matrix(&mut rng, cfg.intermediate_size, cfg.hidden_size),
                down_proj: rand_matrix(&mut rng, cfg.hidden_size, cfg.intermediate_size),
                layer_scalar: 1.0,
                per_layer_input_gate: None,
                per_layer_projection: None,
                post_per_layer_input_norm: None,
            })
            .collect();

        let mut rope_tables = Vec::with_capacity(layers.len());
        for (layer_idx, layer) in layers.iter().enumerate() {
            let head_dim = layer.head_dim(&cfg);
            let rotary_fraction = cfg.layer_rotary_fraction(layer_idx);
            let rope_theta = cfg.layer_rope_theta(layer_idx);
            let max_pos = cfg.max_position_embeddings;
            rope_tables.push(crate::ops::RopeTrigTable::new(
                max_pos,
                head_dim,
                rotary_fraction,
                rope_theta,
            ));
        }

        let instructions = compile_instructions(layers.len());

        Self {
            token_embedding: rand_matrix(&mut rng, cfg.vocab_size, cfg.hidden_size),
            token_embedding_per_layer: None,
            per_layer_model_projection: None,
            per_layer_projection_norm: None,
            per_layer_input_cache: new_per_layer_input_cache(),
            final_norm: rand_norm(&mut rng, cfg.hidden_size),
            lm_head: rand_matrix(&mut rng, cfg.vocab_size, cfg.hidden_size),
            layers,
            lora: None,
            cfg,
            rope_tables,
            instructions,
        }
    }

    pub fn with_lora_adapters(mut self, lora: LoraAdapters) -> Self {
        validate_lora_adapters(&self.cfg, &self.layers, &lora);
        self.lora = Some(lora);
        self
    }

    pub fn load_lora_adapters(&self, adapter_dir: impl AsRef<Path>) -> Result<LoraAdapters> {
        let dims: Vec<_> = self
            .layers
            .iter()
            .map(|layer| LayerLoraDims {
                q_input: self.cfg.hidden_size,
                q_output: layer.q_proj.rows,
                k_input: self.cfg.hidden_size,
                k_output: layer.k_proj.rows,
                v_input: self.cfg.hidden_size,
                v_output: layer.v_proj.rows,
                o_input: layer.o_proj.cols,
                o_output: layer.o_proj.rows,
            })
            .collect();
        load_lora_adapters_from_dir(adapter_dir, &dims)
    }

    pub fn new_cache(&self) -> AnyKvCache {
        self.new_cache_with_capacity(self.cfg.max_position_embeddings)
    }

    pub fn new_cache_with_capacity(&self, max_seq: usize) -> AnyKvCache {
        let layer_shapes: Vec<_> = self
            .layers
            .iter()
            .map(|layer| {
                let head_dim = layer.head_dim(&self.cfg);
                (layer.kv_heads(&self.cfg), head_dim)
            })
            .collect();
        AnyKvCache::Dense(KVCache::new_with_layer_shapes(max_seq, &layer_shapes))
    }

    pub fn forward_token(
        &self,
        token_id: usize,
        position: usize,
        cache: &mut AnyKvCache,
    ) -> Vec<f32> {
        self.forward_token_with_lora(token_id, position, cache, self.lora.as_ref())
    }

    pub fn forward_token_with_lora(
        &self,
        token_id: usize,
        position: usize,
        cache: &mut AnyKvCache,
        active_lora: Option<&LoraAdapters>,
    ) -> Vec<f32> {
        self.forward_token_with_lora_output(token_id, position, cache, active_lora)
            .logits
    }

    pub fn forward_token_with_lora_output(
        &self,
        token_id: usize,
        position: usize,
        cache: &mut AnyKvCache,
        active_lora: Option<&LoraAdapters>,
    ) -> ForwardOutput {
        match cache {
            AnyKvCache::Dense(c) => assert!(position < c.max_seq),
            AnyKvCache::Paged { .. } => {}
        }
        let x: Vec<f32> = self
            .token_embedding
            .row(token_id)
            .iter()
            .map(|v| v * self.cfg.embedding_scale)
            .collect();
        let per_layer_inputs = self.per_layer_inputs_for_token(token_id, &x);
        self.forward_embedding_output(x, position, cache, per_layer_inputs, active_lora)
    }

    pub fn forward_cuneiform_concept(
        &self,
        concept: crate::cuneiform::Concept6D,
        position: usize,
        cache: &mut AnyKvCache,
    ) -> Vec<f32> {
        self.forward_cuneiform_concepts(&[concept], position, cache)
    }

    pub fn forward_cuneiform_concepts(
        &self,
        concepts: &[crate::cuneiform::Concept6D],
        position: usize,
        cache: &mut AnyKvCache,
    ) -> Vec<f32> {
        self.forward_cuneiform_concepts_output(concepts, position, cache)
            .logits
    }

    pub fn forward_cuneiform_concepts_output(
        &self,
        concepts: &[crate::cuneiform::Concept6D],
        position: usize,
        cache: &mut AnyKvCache,
    ) -> ForwardOutput {
        match cache {
            AnyKvCache::Dense(c) => assert!(position < c.max_seq),
            AnyKvCache::Paged { .. } => {}
        }
        let mut x = crate::cuneiform::concepts_embedding(concepts, self.cfg.hidden_size);
        for value in &mut x {
            *value *= self.cfg.embedding_scale;
        }
        let per_layer_inputs = self.per_layer_inputs_for_embedding(&x);
        self.forward_embedding_output(x, position, cache, per_layer_inputs, self.lora.as_ref())
    }

    fn forward_embedding_output(
        &self,
        mut x: Vec<f32>,
        position: usize,
        cache: &mut AnyKvCache,
        per_layer_inputs: Option<PerLayerInputs>,
        active_lora: Option<&LoraAdapters>,
    ) -> ForwardOutput {
        // Reused execution state buffers to avoid allocations
        let mut normed = Vec::new();
        let mut q = Vec::new();
        let mut kv_projection = None;
        let mut attn = Vec::new();
        let mut o_projected = Vec::new();
        let mut hidden = Vec::new();
        let mut down = Vec::new();

        let mut skip_remaining = false;
        let early_exit_threshold = semantic_early_exit_threshold();

        for instr in &self.instructions {
            if skip_remaining {
                break;
            }
            match *instr {
                StaticInstruction::LayerStart { layer_idx: _ } => {}
                StaticInstruction::InputNorm { layer_idx } => {
                    let layer = &self.layers[layer_idx];
                    normed = rms_norm(&x, &layer.input_norm, self.cfg.rms_norm_eps);
                }
                StaticInstruction::ProjQkv { layer_idx } => {
                    let layer = &self.layers[layer_idx];
                    let lora = layer_lora(active_lora, layer_idx);
                    let shared_kv_source = self.cfg.shared_kv_source_layer(layer_idx);
                    let (q_out, kv_proj_out) = if shared_kv_source.is_none() {
                        let (q_v, k_v, v_v) =
                            matvec3(&layer.q_proj, &layer.k_proj, &layer.v_proj, &normed);
                        (q_v, Some((k_v, v_v)))
                    } else {
                        (matvec(&layer.q_proj, &normed), None)
                    };
                    q = q_out;
                    kv_projection = kv_proj_out;
                    apply_lora_if_present(
                        &mut q,
                        &normed,
                        lora.and_then(|layer| layer.q_proj.as_ref()),
                    );
                }
                StaticInstruction::RopeAndCache { layer_idx } => {
                    let layer = &self.layers[layer_idx];
                    let lora = layer_lora(active_lora, layer_idx);
                    let head_dim = layer.head_dim(&self.cfg);
                    let kv_heads = layer.kv_heads(&self.cfg);
                    for head in 0..self.cfg.num_attention_heads {
                        let range = head * head_dim..(head + 1) * head_dim;
                        rms_norm_chunks_in_place(
                            &mut q[range.clone()],
                            head_dim,
                            &layer.q_norm,
                            self.cfg.rms_norm_eps,
                        );
                        apply_rope_split_half_cached(
                            &mut q[range],
                            position,
                            &self.rope_tables[layer_idx],
                        );
                    }
                    if let Some((mut k, mut v)) = kv_projection.take() {
                        apply_lora_if_present(
                            &mut k,
                            &normed,
                            lora.and_then(|layer| layer.k_proj.as_ref()),
                        );
                        apply_lora_if_present(
                            &mut v,
                            &normed,
                            lora.and_then(|layer| layer.v_proj.as_ref()),
                        );
                        for head in 0..kv_heads {
                            let range = head * head_dim..(head + 1) * head_dim;
                            rms_norm_chunks_in_place(
                                &mut k[range.clone()],
                                head_dim,
                                &layer.k_norm,
                                self.cfg.rms_norm_eps,
                            );
                            rms_norm_unit_chunks_in_place(
                                &mut v[range.clone()],
                                head_dim,
                                self.cfg.rms_norm_eps,
                            );
                            apply_rope_split_half_cached(
                                &mut k[range.clone()],
                                position,
                                &self.rope_tables[layer_idx],
                            );
                            cache.set_kv(layer_idx, position, head, &k[range.clone()], &v[range]);
                        }
                    } else if let Some(source_idx) = self.cfg.shared_kv_source_layer(layer_idx) {
                        for head in 0..kv_heads {
                            let k = cache.get_key(source_idx, position, head).to_vec();
                            let v = cache.get_value(source_idx, position, head).to_vec();
                            cache.set_kv(layer_idx, position, head, &k, &v);
                        }
                    }
                }
                StaticInstruction::Attention { layer_idx } => {
                    let layer = &self.layers[layer_idx];
                    let head_dim = layer.head_dim(&self.cfg);
                    let kv_heads = layer.kv_heads(&self.cfg);
                    let shared_kv_source = self.cfg.shared_kv_source_layer(layer_idx);
                    attn = self.attention(
                        AttentionPlan {
                            kv_layer_idx: shared_kv_source.unwrap_or(layer_idx),
                            query_layer_idx: layer_idx,
                            head_dim,
                            kv_heads,
                        },
                        position,
                        &q,
                        cache,
                    );
                }
                StaticInstruction::ProjO { layer_idx } => {
                    let layer = &self.layers[layer_idx];
                    let lora = layer_lora(active_lora, layer_idx);
                    o_projected = matvec(&layer.o_proj, &attn);
                    apply_lora_if_present(
                        &mut o_projected,
                        &attn,
                        lora.and_then(|layer| layer.o_proj.as_ref()),
                    );
                }
                StaticInstruction::AddResidualAttn { layer_idx } => {
                    let layer = &self.layers[layer_idx];
                    rms_norm_in_place(
                        &mut o_projected,
                        &layer.post_attention_norm,
                        self.cfg.rms_norm_eps,
                    );
                    for i in 0..x.len() {
                        x[i] += o_projected[i];
                    }
                }
                StaticInstruction::FeedForwardNorm { layer_idx } => {
                    let layer = &self.layers[layer_idx];
                    normed = rms_norm(&x, &layer.pre_feedforward_norm, self.cfg.rms_norm_eps);
                }
                StaticInstruction::ProjMlpGateUp { layer_idx } => {
                    let layer = &self.layers[layer_idx];
                    let (mut gate, up) = matvec2(&layer.gate_proj, &layer.up_proj, &normed);
                    apply_activation_product_in_place(&self.cfg.hidden_activation, &mut gate, &up);
                    hidden = gate;
                }
                StaticInstruction::ProjMlpDown { layer_idx } => {
                    let layer = &self.layers[layer_idx];
                    down = matvec(&layer.down_proj, &hidden);
                    rms_norm_in_place(
                        &mut down,
                        &layer.post_feedforward_norm,
                        self.cfg.rms_norm_eps,
                    );
                }
                StaticInstruction::AddResidualMlp { layer_idx } => {
                    let layer = &self.layers[layer_idx];
                    for i in 0..x.len() {
                        x[i] += down[i];
                    }
                    if let (
                        Some(per_layer_input),
                        Some(per_layer_input_gate),
                        Some(per_layer_projection),
                        Some(post_per_layer_input_norm),
                    ) = (
                        per_layer_inputs
                            .as_ref()
                            .and_then(|inputs| inputs.get(layer_idx)),
                        layer.per_layer_input_gate.as_ref(),
                        layer.per_layer_projection.as_ref(),
                        layer.post_per_layer_input_norm.as_ref(),
                    ) {
                        let mut gated = matvec(per_layer_input_gate, &x);
                        apply_activation_product_in_place(
                            &self.cfg.hidden_activation,
                            &mut gated,
                            per_layer_input,
                        );
                        let mut proj = matvec(per_layer_projection, &gated);
                        rms_norm_in_place(
                            &mut proj,
                            post_per_layer_input_norm,
                            self.cfg.rms_norm_eps,
                        );
                        for i in 0..x.len() {
                            x[i] += proj[i];
                        }
                    }
                    if layer.layer_scalar != 1.0 {
                        for value in &mut x {
                            *value *= layer.layer_scalar;
                        }
                    }
                }
                StaticInstruction::CheckEarlyExit { layer_idx } => {
                    if let Some(threshold) = early_exit_threshold
                        && should_semantic_early_exit(layer_idx, &x, threshold)
                    {
                        skip_remaining = true;
                    }
                }
            }
        }

        let normed = rms_norm(&x, &self.final_norm, self.cfg.rms_norm_eps);
        let mut logits = matvec(&self.lm_head, &normed);
        if let Some(softcap) = self.cfg.final_logit_softcapping {
            softcap_in_place(&mut logits, softcap);
        }
        ForwardOutput {
            logits,
            hidden_state: normed,
        }
    }

    pub fn forward_batch(
        &self,
        batch: &[(usize, usize)],
        caches: &mut [AnyKvCache],
    ) -> Vec<Vec<f32>> {
        let batch_size = batch.len();
        if batch_size == 0 {
            return Vec::new();
        }

        let mut xs: Vec<Vec<f32>> = batch
            .iter()
            .map(|&(token_id, _)| {
                self.token_embedding
                    .row(token_id)
                    .iter()
                    .map(|v| v * self.cfg.embedding_scale)
                    .collect()
            })
            .collect();

        let per_layer_inputs_batch: Vec<Option<PerLayerInputs>> = batch
            .iter()
            .zip(&xs)
            .map(|(&(token_id, _), x)| self.per_layer_inputs_for_token(token_id, x))
            .collect();

        for (layer_idx, layer) in self.layers.iter().enumerate() {
            let lora = layer_lora(self.lora.as_ref(), layer_idx);
            for i in 0..batch_size {
                let position = batch[i].1;
                let cache = &mut caches[i];
                let mut x = xs[i].clone();
                let normed = rms_norm(&x, &layer.input_norm, self.cfg.rms_norm_eps);
                let head_dim = layer.head_dim(&self.cfg);
                let kv_heads = layer.kv_heads(&self.cfg);
                let shared_kv_source = self.cfg.shared_kv_source_layer(layer_idx);
                let (mut q, kv_projection) = if shared_kv_source.is_none() {
                    let (q, k, v) = matvec3(&layer.q_proj, &layer.k_proj, &layer.v_proj, &normed);
                    (q, Some((k, v)))
                } else {
                    (matvec(&layer.q_proj, &normed), None)
                };
                apply_lora_if_present(
                    &mut q,
                    &normed,
                    lora.and_then(|layer| layer.q_proj.as_ref()),
                );

                for head in 0..self.cfg.num_attention_heads {
                    let range = head * head_dim..(head + 1) * head_dim;
                    rms_norm_chunks_in_place(
                        &mut q[range.clone()],
                        head_dim,
                        &layer.q_norm,
                        self.cfg.rms_norm_eps,
                    );
                    apply_rope_split_half_cached(
                        &mut q[range],
                        position,
                        &self.rope_tables[layer_idx],
                    );
                }

                if let Some((mut k, mut v)) = kv_projection {
                    apply_lora_if_present(
                        &mut k,
                        &normed,
                        lora.and_then(|layer| layer.k_proj.as_ref()),
                    );
                    apply_lora_if_present(
                        &mut v,
                        &normed,
                        lora.and_then(|layer| layer.v_proj.as_ref()),
                    );
                    for head in 0..kv_heads {
                        let range = head * head_dim..(head + 1) * head_dim;
                        rms_norm_chunks_in_place(
                            &mut k[range.clone()],
                            head_dim,
                            &layer.k_norm,
                            self.cfg.rms_norm_eps,
                        );
                        rms_norm_unit_chunks_in_place(
                            &mut v[range.clone()],
                            head_dim,
                            self.cfg.rms_norm_eps,
                        );
                        apply_rope_split_half_cached(
                            &mut k[range.clone()],
                            position,
                            &self.rope_tables[layer_idx],
                        );
                        cache.set_kv(layer_idx, position, head, &k[range.clone()], &v[range]);
                    }
                }

                let attn = self.attention(
                    AttentionPlan {
                        kv_layer_idx: shared_kv_source.unwrap_or(layer_idx),
                        query_layer_idx: layer_idx,
                        head_dim,
                        kv_heads,
                    },
                    position,
                    &q,
                    cache,
                );
                let mut o_projected = matvec(&layer.o_proj, &attn);
                apply_lora_if_present(
                    &mut o_projected,
                    &attn,
                    lora.and_then(|layer| layer.o_proj.as_ref()),
                );
                rms_norm_in_place(
                    &mut o_projected,
                    &layer.post_attention_norm,
                    self.cfg.rms_norm_eps,
                );
                for j in 0..x.len() {
                    x[j] += o_projected[j];
                }

                let normed_mlp = rms_norm(&x, &layer.pre_feedforward_norm, self.cfg.rms_norm_eps);
                let (mut hidden, up) = matvec2(&layer.gate_proj, &layer.up_proj, &normed_mlp);
                apply_activation_product_in_place(&self.cfg.hidden_activation, &mut hidden, &up);
                let mut down = matvec(&layer.down_proj, &hidden);
                rms_norm_in_place(
                    &mut down,
                    &layer.post_feedforward_norm,
                    self.cfg.rms_norm_eps,
                );
                for j in 0..x.len() {
                    x[j] += down[j];
                }

                if let (
                    Some(per_layer_input),
                    Some(per_layer_input_gate),
                    Some(per_layer_projection),
                    Some(post_per_layer_input_norm),
                ) = (
                    per_layer_inputs_batch[i]
                        .as_ref()
                        .and_then(|inputs| inputs.get(layer_idx)),
                    layer.per_layer_input_gate.as_ref(),
                    layer.per_layer_projection.as_ref(),
                    layer.post_per_layer_input_norm.as_ref(),
                ) {
                    let mut gated = matvec(per_layer_input_gate, &x);
                    apply_activation_product_in_place(
                        &self.cfg.hidden_activation,
                        &mut gated,
                        per_layer_input,
                    );
                    let mut projected = matvec(per_layer_projection, &gated);
                    rms_norm_in_place(
                        &mut projected,
                        post_per_layer_input_norm,
                        self.cfg.rms_norm_eps,
                    );
                    for j in 0..x.len() {
                        x[j] += projected[j];
                    }
                }

                if layer.layer_scalar != 1.0 {
                    for value in &mut x {
                        *value *= layer.layer_scalar;
                    }
                }

                xs[i] = x;
            }
        }

        xs.into_iter()
            .map(|x| {
                let normed = rms_norm(&x, &self.final_norm, self.cfg.rms_norm_eps);
                let mut logits = matvec(&self.lm_head, &normed);
                if let Some(softcap) = self.cfg.final_logit_softcapping {
                    softcap_in_place(&mut logits, softcap);
                }
                logits
            })
            .collect()
    }

    pub fn generate_greedy(&self, prompt: &[usize], new_tokens: usize) -> Vec<usize> {
        assert!(!prompt.is_empty());
        let mut rng = StdRng::seed_from_u64(0);
        self.generate_sampled(prompt, new_tokens, SamplingConfig::default(), &mut rng)
    }

    pub fn generate_greedy_with_lora(
        &self,
        prompt: &[usize],
        new_tokens: usize,
        active_lora: Option<&LoraAdapters>,
    ) -> Vec<usize> {
        assert!(!prompt.is_empty());
        let mut rng = StdRng::seed_from_u64(0);
        self.generate_sampled_with_lora(
            prompt,
            new_tokens,
            SamplingConfig::default(),
            &mut rng,
            active_lora,
        )
    }

    pub fn generate_sampled<R: Rng + ?Sized>(
        &self,
        prompt: &[usize],
        new_tokens: usize,
        sampling: SamplingConfig,
        rng: &mut R,
    ) -> Vec<usize> {
        self.generate_sampled_with_lora(prompt, new_tokens, sampling, rng, self.lora.as_ref())
    }

    pub fn generate_sampled_with_lora<R: Rng + ?Sized>(
        &self,
        prompt: &[usize],
        new_tokens: usize,
        sampling: SamplingConfig,
        rng: &mut R,
        active_lora: Option<&LoraAdapters>,
    ) -> Vec<usize> {
        assert!(!prompt.is_empty());
        let mut cache = self.new_cache_with_capacity(prompt.len() + new_tokens + 1);
        let mut out = prompt.to_vec();
        let mut logits = Vec::new();
        for (pos, token_id) in prompt.iter().copied().enumerate() {
            logits = self.forward_token_with_lora(token_id, pos, &mut cache, active_lora);
        }
        for _ in 0..new_tokens {
            let next = sample_next(&logits, sampling, rng);
            out.push(next);
            let pos = out.len() - 1;
            logits = self.forward_token_with_lora(next, pos, &mut cache, active_lora);
        }
        out
    }

    fn attention(
        &self,
        plan: AttentionPlan,
        position: usize,
        q: &[f32],
        cache: &AnyKvCache,
    ) -> Vec<f32> {
        let mut out = vec![0.0; q.len()];
        let scale = 1.0;
        let group = self.cfg.num_attention_heads / plan.kv_heads;
        let start_position = self
            .cfg
            .layer_sliding_window(plan.query_layer_idx)
            .map(|window| (position + 1).saturating_sub(window))
            .unwrap_or(0);

        let use_sparsity = attention_sparsity_enabled();

        for q_head in 0..self.cfg.num_attention_heads {
            let kv_head = q_head / group;
            let q_start = q_head * plan.head_dim;
            let q_vec = &q[q_start..q_start + plan.head_dim];
            let mut scores = Vec::with_capacity(position + 1 - start_position);

            if use_sparsity && position - start_position > 32 {
                let mut downsampled_scores = Vec::with_capacity(position + 1 - start_position);
                let mut max_ds = f32::MIN;
                for t in start_position..=position {
                    let k_vec = cache.get_key(plan.kv_layer_idx, t, kv_head);
                    let mut ds_sum = 0.0;
                    let mut i = 0;
                    while i < plan.head_dim {
                        ds_sum += q_vec[i] * k_vec[i];
                        i += 4;
                    }
                    downsampled_scores.push(ds_sum);
                    if ds_sum > max_ds {
                        max_ds = ds_sum;
                    }
                }

                let ds_threshold = 1.5;
                for (offset, t) in (start_position..=position).enumerate() {
                    let ds_val = downsampled_scores[offset];
                    if ds_val >= max_ds - ds_threshold || t == position {
                        let k_vec = cache.get_key(plan.kv_layer_idx, t, kv_head);
                        scores.push(crate::ops::dot(q_vec, k_vec) * scale);
                    } else {
                        scores.push(-1e9);
                    }
                }
            } else {
                for t in start_position..=position {
                    let k_vec = cache.get_key(plan.kv_layer_idx, t, kv_head);
                    scores.push(crate::ops::dot(q_vec, k_vec) * scale);
                }
            }

            softmax_in_place(&mut scores);
            let out_head = &mut out[q_start..q_start + plan.head_dim];
            for (offset, prob) in scores.iter().enumerate() {
                let t = start_position + offset;
                let v_vec = cache.get_value(plan.kv_layer_idx, t, kv_head);
                for i in 0..plan.head_dim {
                    out_head[i] += prob * v_vec[i];
                }
            }
        }
        out
    }

    fn per_layer_inputs_for_token(
        &self,
        token_id: usize,
        input_embedding: &[f32],
    ) -> Option<PerLayerInputs> {
        let per_layer_dim = self.cfg.hidden_size_per_layer_input?;
        if let Ok(cache) = self.per_layer_input_cache.lock()
            && let Some(cached) = cache.get(&token_id)
        {
            return Some(Arc::clone(cached));
        }
        let mut token_component = self.token_embedding_per_layer.as_ref().map(|embedding| {
            embedding
                .row(token_id)
                .chunks_exact(per_layer_dim)
                .map(|chunk| {
                    chunk
                        .iter()
                        .map(|value| value * self.cfg.per_layer_embedding_scale)
                        .collect::<Vec<_>>()
                })
                .collect::<Vec<_>>()
        });
        let projection_component = match (
            self.per_layer_model_projection.as_ref(),
            self.per_layer_projection_norm.as_ref(),
        ) {
            (Some(projection), Some(norm)) => {
                let mut projected = matvec(projection, input_embedding);
                let scale = (self.cfg.hidden_size as f32).powf(-0.5);
                for value in &mut projected {
                    *value *= scale;
                }
                Some(
                    projected
                        .chunks_exact(per_layer_dim)
                        .map(|chunk| rms_norm(chunk, norm, self.cfg.rms_norm_eps))
                        .collect::<Vec<_>>(),
                )
            }
            _ => None,
        };

        let result = match (token_component.take(), projection_component) {
            (Some(mut token_inputs), Some(projected_inputs)) => {
                let scale = 2.0_f32.powf(-0.5);
                for (token_layer, projected_layer) in token_inputs.iter_mut().zip(projected_inputs)
                {
                    for (token_value, projected_value) in
                        token_layer.iter_mut().zip(projected_layer)
                    {
                        *token_value = (*token_value + projected_value) * scale;
                    }
                }
                Some(token_inputs)
            }
            (Some(token_inputs), None) => Some(token_inputs),
            (None, Some(projected_inputs)) => Some(projected_inputs),
            (None, None) => None,
        };
        let result = result.map(Arc::new);
        if let Some(inputs) = &result
            && let Ok(mut cache) = self.per_layer_input_cache.lock()
            && cache.len() < PER_LAYER_INPUT_CACHE_LIMIT
        {
            cache.entry(token_id).or_insert_with(|| Arc::clone(inputs));
        }
        result
    }

    fn per_layer_inputs_for_embedding(&self, input_embedding: &[f32]) -> Option<PerLayerInputs> {
        let per_layer_dim = self.cfg.hidden_size_per_layer_input?;
        match (
            self.per_layer_model_projection.as_ref(),
            self.per_layer_projection_norm.as_ref(),
        ) {
            (Some(projection), Some(norm)) => {
                let mut projected = matvec(projection, input_embedding);
                let scale = (self.cfg.hidden_size as f32).powf(-0.5);
                for value in &mut projected {
                    *value *= scale;
                }
                Some(Arc::new(
                    projected
                        .chunks_exact(per_layer_dim)
                        .map(|chunk| rms_norm(chunk, norm, self.cfg.rms_norm_eps))
                        .collect::<Vec<_>>(),
                ))
            }
            _ => None,
        }
    }
}

#[derive(Debug, Clone)]
pub struct QuantizedLayer {
    pub input_norm: Vec<f32>,
    pub post_attention_norm: Vec<f32>,
    pub pre_feedforward_norm: Vec<f32>,
    pub post_feedforward_norm: Vec<f32>,
    pub q_norm: Vec<f32>,
    pub k_norm: Vec<f32>,
    pub q_proj: QuantMatrix,
    pub k_proj: QuantMatrix,
    pub v_proj: QuantMatrix,
    pub o_proj: QuantMatrix,
    pub gate_proj: QuantMatrix,
    pub up_proj: QuantMatrix,
    pub down_proj: QuantMatrix,
    pub layer_scalar: f32,
    pub per_layer_input_gate: Option<QuantMatrix>,
    pub per_layer_projection: Option<QuantMatrix>,
    pub post_per_layer_input_norm: Option<Vec<f32>>,
}

pub type QuantizedLayerQ8 = QuantizedLayer;

impl QuantizedLayer {
    pub fn head_dim(&self, cfg: &GemmaConfig) -> usize {
        debug_assert_eq!(self.q_proj.rows() % cfg.num_attention_heads, 0);
        self.q_proj.rows() / cfg.num_attention_heads
    }

    pub fn kv_heads(&self, cfg: &GemmaConfig) -> usize {
        let head_dim = self.head_dim(cfg);
        debug_assert_eq!(self.k_proj.rows() % head_dim, 0);
        self.k_proj.rows() / head_dim
    }
}

#[derive(Debug, Clone)]
pub struct QuantizedGemma {
    pub cfg: GemmaConfig,
    pub activation_mode: QuantizedActivationMode,
    pub token_embedding: QuantMatrix,
    pub token_embedding_per_layer: Option<RowMatrix>,
    pub per_layer_model_projection: Option<QuantMatrix>,
    pub per_layer_projection_norm: Option<Vec<f32>>,
    pub per_layer_input_cache: PerLayerInputCache,
    pub lora: Option<LoraAdapters>,
    pub layers: Vec<QuantizedLayer>,
    pub final_norm: Vec<f32>,
    pub lm_head: QuantMatrix,
    pub rope_tables: Vec<crate::ops::RopeTrigTable>,
    pub instructions: Vec<StaticInstruction>,
    #[cfg(feature = "gpu")]
    gpu_runtime: Option<Arc<crate::gpu::WgpuQ3ModelRuntime>>,
    #[cfg(feature = "gpu")]
    gpu_fallback_warned: Arc<AtomicBool>,
}

pub type QuantizedGemmaQ8 = QuantizedGemma;

impl QuantizedGemma {
    pub fn from_hf_dir(model_dir: impl AsRef<Path>) -> Result<Self> {
        Self::from_hf_dir_inner(model_dir.as_ref(), None, QuantMode::Q8)
    }

    pub fn from_hf_dir_with_cache(
        model_dir: impl AsRef<Path>,
        cache_dir: impl AsRef<Path>,
    ) -> Result<Self> {
        Self::from_hf_dir_inner(model_dir.as_ref(), Some(cache_dir.as_ref()), QuantMode::Q8)
    }

    pub fn from_hf_dir_with_mode(model_dir: impl AsRef<Path>, mode: QuantMode) -> Result<Self> {
        Self::from_hf_dir_inner(model_dir.as_ref(), None, mode)
    }

    pub fn from_hf_dir_with_cache_and_mode(
        model_dir: impl AsRef<Path>,
        cache_dir: impl AsRef<Path>,
        mode: QuantMode,
    ) -> Result<Self> {
        Self::from_hf_dir_inner(model_dir.as_ref(), Some(cache_dir.as_ref()), mode)
    }

    pub fn with_lora_adapters(mut self, lora: LoraAdapters) -> Self {
        validate_quantized_lora_adapters(&self.cfg, &self.layers, &lora);
        self.lora = Some(lora);
        self
    }

    pub fn load_lora_adapters(&self, adapter_dir: impl AsRef<Path>) -> Result<LoraAdapters> {
        let dims: Vec<_> = self
            .layers
            .iter()
            .map(|layer| LayerLoraDims {
                q_input: self.cfg.hidden_size,
                q_output: layer.q_proj.rows(),
                k_input: self.cfg.hidden_size,
                k_output: layer.k_proj.rows(),
                v_input: self.cfg.hidden_size,
                v_output: layer.v_proj.rows(),
                o_input: layer.o_proj.cols(),
                o_output: layer.o_proj.rows(),
            })
            .collect();
        load_lora_adapters_from_dir(adapter_dir, &dims)
    }

    pub fn with_activation_mode(mut self, activation_mode: QuantizedActivationMode) -> Self {
        self.activation_mode = activation_mode;
        self
    }

    fn from_hf_dir_inner(
        model_dir: &Path,
        q8_cache_dir: Option<&Path>,
        mode: QuantMode,
    ) -> Result<Self> {
        Self::from_source_inner(ModelSource::Dir(model_dir), q8_cache_dir, mode)
    }

    pub fn from_source_inner(
        source: ModelSource<'_>,
        q8_cache_dir: Option<&Path>,
        mode: QuantMode,
    ) -> Result<Self> {
        let resolution = match source {
            ModelSource::Dir(model_dir) => crate::gemma_hf::resolve_gemma_dir(model_dir)
                .with_context(|| format!("resolving Gemma model at {}", model_dir.display()))?,
            ModelSource::InMemory { config_json, files } => {
                let index = crate::weights::TensorIndex::from_in_memory(files)?;
                crate::gemma_hf::resolve_gemma_in_memory(config_json, &index)?
            }
        };
        let cfg = resolution.config;
        cfg.validate();

        let mut reader = match source {
            ModelSource::Dir(model_dir) => crate::weights::TensorReader::from_dir(model_dir)
                .with_context(|| format!("indexing safetensors in {}", model_dir.display()))?,
            ModelSource::InMemory { files, .. } => {
                crate::weights::TensorReader::from_in_memory(files.clone())?
            }
        };

        let roles: HashMap<_, _> = resolution
            .tensors
            .iter()
            .filter_map(|tensor| {
                tensor
                    .name
                    .as_ref()
                    .map(|name| (tensor.role.as_str(), name.as_str()))
            })
            .collect();

        let token_embedding_name = required_role(&roles, "token_embedding")?;
        let token_embedding = load_quant_matrix_cached(
            source,
            &mut reader,
            token_embedding_name,
            cfg.vocab_size,
            cfg.hidden_size,
            "token_embedding",
            mode,
            q8_cache_dir,
        )?;
        let token_embedding_per_layer = if let Some(name) =
            roles.get("token_embedding_per_layer").copied()
        {
            let per_layer_dim = cfg.hidden_size_per_layer_input.with_context(|| {
                "token_embedding_per_layer tensor found but config lacks hidden_size_per_layer_input"
            })?;
            let table = LazyRowTensor::from_reader(&mut reader, name).with_context(|| {
                format!("mapping tensor role=token_embedding_per_layer name={name}")
            })?;
            let expected_rows = cfg.vocab_size_per_layer_input.unwrap_or(cfg.vocab_size);
            let expected_cols = cfg.num_hidden_layers * per_layer_dim;
            if table.rows() != expected_rows || table.cols() != expected_cols {
                bail!(
                    "shape mismatch for role=token_embedding_per_layer name={name}: expected [{expected_rows}, {expected_cols}] got [{}, {}]",
                    table.rows(),
                    table.cols()
                );
            }
            Some(RowMatrix::Lazy(table))
        } else {
            None
        };
        let per_layer_model_projection = if let Some(name) =
            roles.get("per_layer_model_projection").copied()
        {
            let per_layer_dim = cfg.hidden_size_per_layer_input.with_context(|| {
                    "per_layer_model_projection tensor found but config lacks hidden_size_per_layer_input"
                })?;
            Some(load_quant_matrix_cached(
                source,
                &mut reader,
                name,
                cfg.num_hidden_layers * per_layer_dim,
                cfg.hidden_size,
                "per_layer_model_projection",
                mode,
                q8_cache_dir,
            )?)
        } else {
            None
        };
        let per_layer_projection_norm = if let Some(name) =
            roles.get("per_layer_projection_norm").copied()
        {
            Some(load_norm(
                    &mut reader,
                    name,
                    cfg.hidden_size_per_layer_input.with_context(|| {
                        "per_layer_projection_norm tensor found but config lacks hidden_size_per_layer_input"
                    })?,
                    "per_layer_projection_norm",
                    cfg.fold_rms_norm,
                )?)
        } else {
            None
        };
        let final_norm = load_norm(
            &mut reader,
            required_role(&roles, "final_norm")?,
            cfg.hidden_size,
            "final_norm",
            cfg.fold_rms_norm,
        )?;

        let lm_head = if let Some(name) = roles.get("lm_head").copied() {
            if name == token_embedding_name {
                token_embedding.clone()
            } else {
                load_quant_matrix_cached(
                    source,
                    &mut reader,
                    name,
                    cfg.vocab_size,
                    cfg.hidden_size,
                    "lm_head",
                    mode,
                    q8_cache_dir,
                )?
            }
        } else {
            token_embedding.clone()
        };

        let mut layers = Vec::with_capacity(cfg.num_hidden_layers);
        for layer in 0..cfg.num_hidden_layers {
            let role = |suffix: &str| format!("layers.{layer}.{suffix}");
            let input_norm = load_norm(
                &mut reader,
                required_role(&roles, &role("input_norm"))?,
                cfg.hidden_size,
                &role("input_norm"),
                cfg.fold_rms_norm,
            )?;
            let post_attention_norm = load_norm(
                &mut reader,
                required_role(&roles, &role("post_attention_norm"))?,
                cfg.hidden_size,
                &role("post_attention_norm"),
                cfg.fold_rms_norm,
            )?;
            let pre_feedforward_norm = load_optional_norm(
                &mut reader,
                roles.get(role("pre_feedforward_norm").as_str()).copied(),
                cfg.hidden_size,
                &role("pre_feedforward_norm"),
                cfg.fold_rms_norm,
            )?
            .unwrap_or_else(|| post_attention_norm.clone());
            let post_feedforward_norm = load_optional_norm(
                &mut reader,
                roles.get(role("post_feedforward_norm").as_str()).copied(),
                cfg.hidden_size,
                &role("post_feedforward_norm"),
                cfg.fold_rms_norm,
            )?
            .unwrap_or_else(|| vec![1.0; cfg.hidden_size]);
            let q_proj = load_quant_matrix_with_cols_cached(
                source,
                &mut reader,
                required_role(&roles, &role("q_proj"))?,
                cfg.hidden_size,
                &role("q_proj"),
                mode,
                q8_cache_dir,
            )?;
            if q_proj.rows() % cfg.num_attention_heads != 0 {
                bail!(
                    "shape mismatch for role={}: q rows {} are not divisible by attention heads {}",
                    role("q_proj"),
                    q_proj.rows(),
                    cfg.num_attention_heads
                );
            }
            let layer_head_dim = q_proj.rows() / cfg.num_attention_heads;
            let k_proj = load_quant_matrix_with_cols_cached(
                source,
                &mut reader,
                required_role(&roles, &role("k_proj"))?,
                cfg.hidden_size,
                &role("k_proj"),
                mode,
                q8_cache_dir,
            )?;
            let v_proj = load_quant_matrix_with_cols_cached(
                source,
                &mut reader,
                required_role(&roles, &role("v_proj"))?,
                cfg.hidden_size,
                &role("v_proj"),
                mode,
                q8_cache_dir,
            )?;
            if k_proj.rows() != v_proj.rows() || k_proj.rows() % layer_head_dim != 0 {
                bail!(
                    "shape mismatch for layer {layer}: k/v rows {} / {} not compatible with head_dim {}",
                    k_proj.rows(),
                    v_proj.rows(),
                    layer_head_dim
                );
            }
            let o_proj = load_quant_matrix_cached(
                source,
                &mut reader,
                required_role(&roles, &role("o_proj"))?,
                cfg.hidden_size,
                q_proj.rows(),
                &role("o_proj"),
                mode,
                q8_cache_dir,
            )?;
            let q_norm = load_optional_norm(
                &mut reader,
                roles.get(role("q_norm").as_str()).copied(),
                layer_head_dim,
                &role("q_norm"),
                cfg.fold_rms_norm,
            )?
            .unwrap_or_else(|| vec![1.0; layer_head_dim]);
            let k_norm = load_optional_norm(
                &mut reader,
                roles.get(role("k_norm").as_str()).copied(),
                layer_head_dim,
                &role("k_norm"),
                cfg.fold_rms_norm,
            )?
            .unwrap_or_else(|| vec![1.0; layer_head_dim]);
            let gate_proj = load_quant_matrix_with_cols_cached(
                source,
                &mut reader,
                required_role(&roles, &role("gate_proj"))?,
                cfg.hidden_size,
                &role("gate_proj"),
                mode,
                q8_cache_dir,
            )?;
            let up_proj = load_quant_matrix_with_cols_cached(
                source,
                &mut reader,
                required_role(&roles, &role("up_proj"))?,
                cfg.hidden_size,
                &role("up_proj"),
                mode,
                q8_cache_dir,
            )?;
            if up_proj.rows() != gate_proj.rows() {
                bail!(
                    "shape mismatch for layer {layer}: gate rows {} != up rows {}",
                    gate_proj.rows(),
                    up_proj.rows()
                );
            }
            let down_proj = load_quant_matrix_cached(
                source,
                &mut reader,
                required_role(&roles, &role("down_proj"))?,
                cfg.hidden_size,
                gate_proj.rows(),
                &role("down_proj"),
                mode,
                q8_cache_dir,
            )?;
            let layer_scalar = load_optional_vec(
                &mut reader,
                roles.get(role("layer_scalar").as_str()).copied(),
                1,
                &role("layer_scalar"),
            )?
            .map(|v| v[0])
            .unwrap_or(1.0);
            let per_layer_input_gate =
                if let Some(name) = roles.get(role("per_layer_input_gate").as_str()).copied() {
                    Some(load_quant_matrix_with_cols_cached(
                        source,
                        &mut reader,
                        name,
                        cfg.hidden_size,
                        &role("per_layer_input_gate"),
                        mode,
                        q8_cache_dir,
                    )?)
                } else {
                    None
                };
            let per_layer_projection =
                if let Some(name) = roles.get(role("per_layer_projection").as_str()).copied() {
                    let input_dim = per_layer_input_gate
                    .as_ref()
                    .map(|m| m.rows())
                    .or(cfg.hidden_size_per_layer_input)
                    .with_context(|| {
                        format!(
                            "{} present but no per-layer input gate or hidden_size_per_layer_input",
                            role("per_layer_projection")
                        )
                    })?;
                    Some(load_quant_matrix_cached(
                        source,
                        &mut reader,
                        name,
                        cfg.hidden_size,
                        input_dim,
                        &role("per_layer_projection"),
                        mode,
                        q8_cache_dir,
                    )?)
                } else {
                    None
                };
            let post_per_layer_input_norm = load_optional_norm(
                &mut reader,
                roles
                    .get(role("post_per_layer_input_norm").as_str())
                    .copied(),
                cfg.hidden_size,
                &role("post_per_layer_input_norm"),
                cfg.fold_rms_norm,
            )?;

            layers.push(QuantizedLayer {
                input_norm,
                post_attention_norm,
                pre_feedforward_norm,
                post_feedforward_norm,
                q_norm,
                k_norm,
                q_proj,
                k_proj,
                v_proj,
                o_proj,
                gate_proj,
                up_proj,
                down_proj,
                layer_scalar,
                per_layer_input_gate,
                per_layer_projection,
                post_per_layer_input_norm,
            });
        }

        if let Some(dir) = q8_cache_dir {
            write_manifest_for_cache(dir, mode, &cfg)?;
        }

        let mut rope_tables = Vec::with_capacity(layers.len());
        for (layer_idx, layer) in layers.iter().enumerate() {
            let head_dim = layer.head_dim(&cfg);
            let rotary_fraction = cfg.layer_rotary_fraction(layer_idx);
            let rope_theta = cfg.layer_rope_theta(layer_idx);
            let max_pos = cfg.max_position_embeddings;
            rope_tables.push(crate::ops::RopeTrigTable::new(
                max_pos,
                head_dim,
                rotary_fraction,
                rope_theta,
            ));
        }

        let instructions = compile_instructions(layers.len());

        Ok(Self {
            cfg,
            activation_mode: QuantizedActivationMode::F32,
            token_embedding,
            token_embedding_per_layer,
            per_layer_model_projection,
            per_layer_projection_norm,
            per_layer_input_cache: new_per_layer_input_cache(),
            layers,
            lora: None,
            final_norm,
            lm_head,
            rope_tables,
            instructions,
            #[cfg(feature = "gpu")]
            gpu_runtime: None,
            #[cfg(feature = "gpu")]
            gpu_fallback_warned: Arc::new(AtomicBool::new(false)),
        })
    }

    pub fn from_native(model: &NativeGemma) -> Self {
        Self::from_native_with_mode(model, QuantMode::Q8)
    }

    pub fn from_native_with_mode(model: &NativeGemma, mode: QuantMode) -> Self {
        let quant = |matrix: &Matrix| match mode {
            QuantMode::Q8 => QuantMatrix::Q8Resident(RowQ8Matrix::quantize(matrix)),
            QuantMode::Q5 => QuantMatrix::Q5Resident(RowQ5Matrix::quantize(matrix)),
            QuantMode::Q4 => QuantMatrix::Q4Resident(RowQ4Matrix::quantize(matrix)),
            QuantMode::Q3 | QuantMode::Q1_58 => {
                QuantMatrix::Q3Resident(crate::quant::RowQ3Matrix::quantize(matrix))
            }
        };

        Self {
            cfg: model.cfg.clone(),
            activation_mode: QuantizedActivationMode::F32,
            token_embedding: quant(&model.token_embedding),
            token_embedding_per_layer: model.token_embedding_per_layer.clone(),
            per_layer_model_projection: model.per_layer_model_projection.as_ref().map(&quant),
            per_layer_projection_norm: model.per_layer_projection_norm.clone(),
            per_layer_input_cache: new_per_layer_input_cache(),
            layers: model
                .layers
                .iter()
                .map(|layer| QuantizedLayer {
                    input_norm: layer.input_norm.clone(),
                    post_attention_norm: layer.post_attention_norm.clone(),
                    pre_feedforward_norm: layer.pre_feedforward_norm.clone(),
                    post_feedforward_norm: layer.post_feedforward_norm.clone(),
                    q_norm: layer.q_norm.clone(),
                    k_norm: layer.k_norm.clone(),
                    q_proj: quant(&layer.q_proj),
                    k_proj: quant(&layer.k_proj),
                    v_proj: quant(&layer.v_proj),
                    o_proj: quant(&layer.o_proj),
                    gate_proj: quant(&layer.gate_proj),
                    up_proj: quant(&layer.up_proj),
                    down_proj: quant(&layer.down_proj),
                    layer_scalar: layer.layer_scalar,
                    per_layer_input_gate: layer.per_layer_input_gate.as_ref().map(&quant),
                    per_layer_projection: layer.per_layer_projection.as_ref().map(&quant),
                    post_per_layer_input_norm: layer.post_per_layer_input_norm.clone(),
                })
                .collect(),
            lora: model.lora.clone(),
            final_norm: model.final_norm.clone(),
            lm_head: quant(&model.lm_head),
            rope_tables: model.rope_tables.clone(),
            instructions: model.instructions.clone(),
            #[cfg(feature = "gpu")]
            gpu_runtime: None,
            #[cfg(feature = "gpu")]
            gpu_fallback_warned: Arc::new(AtomicBool::new(false)),
        }
    }

    #[cfg(feature = "gpu")]
    pub fn with_q3_gpu(mut self) -> Result<Self> {
        let backend =
            crate::gpu::WgpuMatvecBackend::new().context("initializing Q3 GPU backend")?;
        let mut runtime = {
            let matrices = self.q3_gpu_matrices();
            let uploads = matrices
                .iter()
                .enumerate()
                .map(|(index, matrix)| {
                    matrix.q3_gpu_upload().with_context(|| {
                        format!("matrix {index} is not compatible with the packed Q3 GPU runtime")
                    })
                })
                .collect::<Result<Vec<_>>>()?;
            backend
                .prepare_q3_model(&uploads)
                .context("uploading packed Q3 model to GPU")?
        };
        let mlp_triples =
            self.layers
                .iter()
                .enumerate()
                .map(|(layer_idx, layer)| {
                    let gate = layer.gate_proj.q3_gpu_descriptor().with_context(|| {
                        format!("layer {layer_idx} gate projection is not packed Q3")
                    })?;
                    let up = layer.up_proj.q3_gpu_descriptor().with_context(|| {
                        format!("layer {layer_idx} up projection is not packed Q3")
                    })?;
                    let down = layer.down_proj.q3_gpu_descriptor().with_context(|| {
                        format!("layer {layer_idx} down projection is not packed Q3")
                    })?;
                    Ok((gate.0, up.0, down.0))
                })
                .collect::<Result<Vec<_>>>()?;
        runtime
            .prepare_mlp_plans(&mlp_triples, &self.cfg.hidden_activation)
            .context("preparing fused Q3 GPU MLP plans")?;
        eprintln!(
            "zymatica Q3 GPU ready: adapter={} backend={} matrices={} fused_mlp_plans={} resident_mb={:.2}",
            runtime.info().adapter_name,
            runtime.info().backend,
            runtime.matrix_count(),
            runtime.mlp_plan_count(),
            runtime.resident_bytes() as f64 / (1024.0 * 1024.0)
        );
        self.activation_mode = QuantizedActivationMode::GpuF32;
        self.gpu_runtime = Some(Arc::new(runtime));
        Ok(self)
    }

    #[cfg(not(feature = "gpu"))]
    pub fn with_q3_gpu(self) -> Result<Self> {
        bail!("Q3 GPU execution requires building zymatica-engine with the 'gpu' feature")
    }

    #[cfg(feature = "gpu")]
    fn q3_gpu_matrices(&self) -> Vec<&QuantMatrix> {
        let mut matrices = Vec::with_capacity(self.layers.len() * 9 + 2);
        if let Some(matrix) = self.per_layer_model_projection.as_ref() {
            matrices.push(matrix);
        }
        for layer in &self.layers {
            matrices.extend([
                &layer.q_proj,
                &layer.k_proj,
                &layer.v_proj,
                &layer.o_proj,
                &layer.gate_proj,
                &layer.up_proj,
                &layer.down_proj,
            ]);
            if let Some(matrix) = layer.per_layer_input_gate.as_ref() {
                matrices.push(matrix);
            }
            if let Some(matrix) = layer.per_layer_projection.as_ref() {
                matrices.push(matrix);
            }
        }
        matrices.push(&self.lm_head);
        matrices
    }

    #[cfg(feature = "gpu")]
    fn warn_gpu_fallback(&self, error: &anyhow::Error) {
        if !self.gpu_fallback_warned.swap(true, Ordering::Relaxed) {
            eprintln!("warning: Q3 GPU execution failed; falling back to CPU: {error:#}");
        }
    }

    #[cfg(feature = "gpu")]
    fn gpu_mlp(&self, layer: &QuantizedLayer, x: &[f32]) -> Option<Vec<f32>> {
        if self.activation_mode != QuantizedActivationMode::GpuF32 {
            return None;
        }
        let runtime = self.gpu_runtime.as_ref()?;
        let gate = layer.gate_proj.q3_gpu_descriptor()?;
        let up = layer.up_proj.q3_gpu_descriptor()?;
        let down = layer.down_proj.q3_gpu_descriptor()?;
        match runtime.matvec_mlp(gate, up, down, x) {
            Ok(output) => Some(output),
            Err(error) => {
                self.warn_gpu_fallback(&error);
                None
            }
        }
    }

    #[cfg(not(feature = "gpu"))]
    fn gpu_mlp(&self, _layer: &QuantizedLayer, _x: &[f32]) -> Option<Vec<f32>> {
        None
    }

    fn matvec(&self, matrix: &QuantMatrix, x: &[f32]) -> Vec<f32> {
        #[cfg(feature = "gpu")]
        if self.activation_mode == QuantizedActivationMode::GpuF32
            && let (Some(runtime), Some((key, rows, cols))) =
                (&self.gpu_runtime, matrix.q3_gpu_descriptor())
        {
            match runtime.matvec(key, rows, cols, x) {
                Ok(output) => return output,
                Err(error) => self.warn_gpu_fallback(&error),
            }
        }
        matrix.matvec_with_activation_mode(x, self.activation_mode)
    }

    fn matvec2(&self, a: &QuantMatrix, b: &QuantMatrix, x: &[f32]) -> (Vec<f32>, Vec<f32>) {
        #[cfg(feature = "gpu")]
        if self.activation_mode == QuantizedActivationMode::GpuF32
            && let (Some(runtime), Some(a_desc), Some(b_desc)) = (
                &self.gpu_runtime,
                a.q3_gpu_descriptor(),
                b.q3_gpu_descriptor(),
            )
        {
            match runtime.matvec2(a_desc, b_desc, x) {
                Ok(output) => return output,
                Err(error) => self.warn_gpu_fallback(&error),
            }
        }
        QuantMatrix::matvec2_with_activation_mode(a, b, x, self.activation_mode)
    }

    fn matvec3(
        &self,
        a: &QuantMatrix,
        b: &QuantMatrix,
        c: &QuantMatrix,
        x: &[f32],
    ) -> (Vec<f32>, Vec<f32>, Vec<f32>) {
        #[cfg(feature = "gpu")]
        if self.activation_mode == QuantizedActivationMode::GpuF32
            && let (Some(runtime), Some(a_desc), Some(b_desc), Some(c_desc)) = (
                &self.gpu_runtime,
                a.q3_gpu_descriptor(),
                b.q3_gpu_descriptor(),
                c.q3_gpu_descriptor(),
            )
        {
            match runtime.matvec3(a_desc, b_desc, c_desc, x) {
                Ok(output) => return output,
                Err(error) => self.warn_gpu_fallback(&error),
            }
        }
        QuantMatrix::matvec3_with_activation_mode(a, b, c, x, self.activation_mode)
    }

    pub fn new_cache(&self) -> AnyKvCache {
        self.new_cache_with_capacity(self.cfg.max_position_embeddings)
    }

    pub fn new_cache_with_capacity(&self, max_seq: usize) -> AnyKvCache {
        let layer_shapes: Vec<_> = self
            .layers
            .iter()
            .map(|layer| {
                let head_dim = layer.head_dim(&self.cfg);
                (layer.kv_heads(&self.cfg), head_dim)
            })
            .collect();
        AnyKvCache::Dense(KVCache::new_with_layer_shapes(max_seq, &layer_shapes))
    }

    pub fn forward_token(
        &self,
        token_id: usize,
        position: usize,
        cache: &mut AnyKvCache,
    ) -> Vec<f32> {
        self.forward_token_with_lora(token_id, position, cache, self.lora.as_ref())
    }

    pub fn forward_token_with_lora(
        &self,
        token_id: usize,
        position: usize,
        cache: &mut AnyKvCache,
        active_lora: Option<&LoraAdapters>,
    ) -> Vec<f32> {
        self.forward_token_with_lora_output(token_id, position, cache, active_lora)
            .logits
    }

    pub fn forward_token_with_lora_output(
        &self,
        token_id: usize,
        position: usize,
        cache: &mut AnyKvCache,
        active_lora: Option<&LoraAdapters>,
    ) -> ForwardOutput {
        match cache {
            AnyKvCache::Dense(c) => assert!(position < c.max_seq),
            AnyKvCache::Paged { .. } => {}
        }
        let mut x = self.token_embedding.dequantize_row(token_id);
        for value in &mut x {
            *value *= self.cfg.embedding_scale;
        }
        let per_layer_inputs = self.per_layer_inputs_for_token(token_id, &x);
        self.forward_embedding_output(x, position, cache, per_layer_inputs, active_lora)
    }

    pub fn forward_cuneiform_concept(
        &self,
        concept: crate::cuneiform::Concept6D,
        position: usize,
        cache: &mut AnyKvCache,
    ) -> Vec<f32> {
        self.forward_cuneiform_concepts(&[concept], position, cache)
    }

    pub fn forward_cuneiform_concepts(
        &self,
        concepts: &[crate::cuneiform::Concept6D],
        position: usize,
        cache: &mut AnyKvCache,
    ) -> Vec<f32> {
        self.forward_cuneiform_concepts_output(concepts, position, cache)
            .logits
    }

    pub fn forward_cuneiform_concepts_output(
        &self,
        concepts: &[crate::cuneiform::Concept6D],
        position: usize,
        cache: &mut AnyKvCache,
    ) -> ForwardOutput {
        match cache {
            AnyKvCache::Dense(c) => assert!(position < c.max_seq),
            AnyKvCache::Paged { .. } => {}
        }
        let mut x = crate::cuneiform::concepts_embedding(concepts, self.cfg.hidden_size);
        for value in &mut x {
            *value *= self.cfg.embedding_scale;
        }
        let per_layer_inputs = self.per_layer_inputs_for_embedding(&x);
        self.forward_embedding_output(x, position, cache, per_layer_inputs, self.lora.as_ref())
    }

    fn forward_embedding_output(
        &self,
        mut x: Vec<f32>,
        position: usize,
        cache: &mut AnyKvCache,
        per_layer_inputs: Option<PerLayerInputs>,
        active_lora: Option<&LoraAdapters>,
    ) -> ForwardOutput {
        // Reused execution state buffers to avoid allocations
        let mut normed = Vec::new();
        let mut q = Vec::new();
        let mut kv_projection = None;
        let mut attn = Vec::new();
        let mut o_projected = Vec::new();
        let mut hidden = Vec::new();
        let mut down = Vec::new();
        let mut gpu_mlp_ready = false;

        let mut skip_remaining = false;
        let early_exit_threshold = semantic_early_exit_threshold();

        for instr in &self.instructions {
            if skip_remaining {
                break;
            }
            match *instr {
                StaticInstruction::LayerStart { layer_idx: _ } => {}
                StaticInstruction::InputNorm { layer_idx } => {
                    let layer = &self.layers[layer_idx];
                    normed = rms_norm(&x, &layer.input_norm, self.cfg.rms_norm_eps);
                }
                StaticInstruction::ProjQkv { layer_idx } => {
                    let layer = &self.layers[layer_idx];
                    let lora = layer_lora(active_lora, layer_idx);
                    let shared_kv_source = self.cfg.shared_kv_source_layer(layer_idx);
                    let (q_out, kv_proj_out) = if shared_kv_source.is_none() {
                        let (q_v, k_v, v_v) =
                            self.matvec3(&layer.q_proj, &layer.k_proj, &layer.v_proj, &normed);
                        (q_v, Some((k_v, v_v)))
                    } else {
                        (self.matvec(&layer.q_proj, &normed), None)
                    };
                    q = q_out;
                    kv_projection = kv_proj_out;
                    apply_lora_if_present(
                        &mut q,
                        &normed,
                        lora.and_then(|layer| layer.q_proj.as_ref()),
                    );
                }
                StaticInstruction::RopeAndCache { layer_idx } => {
                    let layer = &self.layers[layer_idx];
                    let lora = layer_lora(active_lora, layer_idx);
                    let head_dim = layer.head_dim(&self.cfg);
                    let kv_heads = layer.kv_heads(&self.cfg);
                    for head in 0..self.cfg.num_attention_heads {
                        let range = head * head_dim..(head + 1) * head_dim;
                        rms_norm_chunks_in_place(
                            &mut q[range.clone()],
                            head_dim,
                            &layer.q_norm,
                            self.cfg.rms_norm_eps,
                        );
                        apply_rope_split_half_cached(
                            &mut q[range],
                            position,
                            &self.rope_tables[layer_idx],
                        );
                    }
                    if let Some((mut k, mut v)) = kv_projection.take() {
                        apply_lora_if_present(
                            &mut k,
                            &normed,
                            lora.and_then(|layer| layer.k_proj.as_ref()),
                        );
                        apply_lora_if_present(
                            &mut v,
                            &normed,
                            lora.and_then(|layer| layer.v_proj.as_ref()),
                        );
                        for head in 0..kv_heads {
                            let range = head * head_dim..(head + 1) * head_dim;
                            rms_norm_chunks_in_place(
                                &mut k[range.clone()],
                                head_dim,
                                &layer.k_norm,
                                self.cfg.rms_norm_eps,
                            );
                            rms_norm_unit_chunks_in_place(
                                &mut v[range.clone()],
                                head_dim,
                                self.cfg.rms_norm_eps,
                            );
                            apply_rope_split_half_cached(
                                &mut k[range.clone()],
                                position,
                                &self.rope_tables[layer_idx],
                            );
                            cache.set_kv(layer_idx, position, head, &k[range.clone()], &v[range]);
                        }
                    } else if let Some(source_idx) = self.cfg.shared_kv_source_layer(layer_idx) {
                        for head in 0..kv_heads {
                            let k = cache.get_key(source_idx, position, head).to_vec();
                            let v = cache.get_value(source_idx, position, head).to_vec();
                            cache.set_kv(layer_idx, position, head, &k, &v);
                        }
                    }
                }
                StaticInstruction::Attention { layer_idx } => {
                    let layer = &self.layers[layer_idx];
                    let head_dim = layer.head_dim(&self.cfg);
                    let kv_heads = layer.kv_heads(&self.cfg);
                    let shared_kv_source = self.cfg.shared_kv_source_layer(layer_idx);
                    attn = self.attention(
                        AttentionPlan {
                            kv_layer_idx: shared_kv_source.unwrap_or(layer_idx),
                            query_layer_idx: layer_idx,
                            head_dim,
                            kv_heads,
                        },
                        position,
                        &q,
                        cache,
                    );
                }
                StaticInstruction::ProjO { layer_idx } => {
                    let layer = &self.layers[layer_idx];
                    let lora = layer_lora(active_lora, layer_idx);
                    o_projected = self.matvec(&layer.o_proj, &attn);
                    apply_lora_if_present(
                        &mut o_projected,
                        &attn,
                        lora.and_then(|layer| layer.o_proj.as_ref()),
                    );
                }
                StaticInstruction::AddResidualAttn { layer_idx } => {
                    let layer = &self.layers[layer_idx];
                    rms_norm_in_place(
                        &mut o_projected,
                        &layer.post_attention_norm,
                        self.cfg.rms_norm_eps,
                    );
                    for i in 0..x.len() {
                        x[i] += o_projected[i];
                    }
                }
                StaticInstruction::FeedForwardNorm { layer_idx } => {
                    let layer = &self.layers[layer_idx];
                    normed = rms_norm(&x, &layer.pre_feedforward_norm, self.cfg.rms_norm_eps);
                }
                StaticInstruction::ProjMlpGateUp { layer_idx } => {
                    let layer = &self.layers[layer_idx];
                    if let Some(gpu_down) = self.gpu_mlp(layer, &normed) {
                        down = gpu_down;
                        gpu_mlp_ready = true;
                    } else {
                        let (mut gate, up) =
                            self.matvec2(&layer.gate_proj, &layer.up_proj, &normed);
                        apply_activation_product_in_place(
                            &self.cfg.hidden_activation,
                            &mut gate,
                            &up,
                        );
                        hidden = gate;
                        gpu_mlp_ready = false;
                    }
                }
                StaticInstruction::ProjMlpDown { layer_idx } => {
                    let layer = &self.layers[layer_idx];
                    if !gpu_mlp_ready {
                        down = self.matvec(&layer.down_proj, &hidden);
                    }
                    rms_norm_in_place(
                        &mut down,
                        &layer.post_feedforward_norm,
                        self.cfg.rms_norm_eps,
                    );
                }
                StaticInstruction::AddResidualMlp { layer_idx } => {
                    let layer = &self.layers[layer_idx];
                    for i in 0..x.len() {
                        x[i] += down[i];
                    }
                    if let (
                        Some(per_layer_input),
                        Some(per_layer_input_gate),
                        Some(per_layer_projection),
                        Some(post_per_layer_input_norm),
                    ) = (
                        per_layer_inputs
                            .as_ref()
                            .and_then(|inputs| inputs.get(layer_idx)),
                        layer.per_layer_input_gate.as_ref(),
                        layer.per_layer_projection.as_ref(),
                        layer.post_per_layer_input_norm.as_ref(),
                    ) {
                        let mut gated = self.matvec(per_layer_input_gate, &x);
                        apply_activation_product_in_place(
                            &self.cfg.hidden_activation,
                            &mut gated,
                            per_layer_input,
                        );
                        let mut proj = self.matvec(per_layer_projection, &gated);
                        rms_norm_in_place(
                            &mut proj,
                            post_per_layer_input_norm,
                            self.cfg.rms_norm_eps,
                        );
                        for i in 0..x.len() {
                            x[i] += proj[i];
                        }
                    }
                    if layer.layer_scalar != 1.0 {
                        for value in &mut x {
                            *value *= layer.layer_scalar;
                        }
                    }
                }
                StaticInstruction::CheckEarlyExit { layer_idx } => {
                    if let Some(threshold) = early_exit_threshold
                        && should_semantic_early_exit(layer_idx, &x, threshold)
                    {
                        skip_remaining = true;
                    }
                }
            }
        }

        let x_normed = rms_norm(&x, &self.final_norm, self.cfg.rms_norm_eps);
        let mut logits = self.matvec(&self.lm_head, &x_normed);
        if let Some(softcap) = self.cfg.final_logit_softcapping {
            softcap_in_place(&mut logits, softcap);
        }
        ForwardOutput {
            logits,
            hidden_state: x_normed,
        }
    }

    pub fn forward_batch(
        &self,
        batch: &[(usize, usize)],
        caches: &mut [AnyKvCache],
    ) -> Vec<Vec<f32>> {
        let batch_size = batch.len();
        if batch_size == 0 {
            return Vec::new();
        }

        let mut xs: Vec<Vec<f32>> = batch
            .iter()
            .map(|&(token_id, _)| {
                let mut x = self.token_embedding.dequantize_row(token_id);
                for value in &mut x {
                    *value *= self.cfg.embedding_scale;
                }
                x
            })
            .collect();

        let per_layer_inputs_batch: Vec<Option<PerLayerInputs>> = batch
            .iter()
            .zip(&xs)
            .map(|(&(token_id, _), x)| self.per_layer_inputs_for_token(token_id, x))
            .collect();

        for (layer_idx, layer) in self.layers.iter().enumerate() {
            let lora = layer_lora(self.lora.as_ref(), layer_idx);
            for i in 0..batch_size {
                let position = batch[i].1;
                let cache = &mut caches[i];
                let mut x = xs[i].clone();
                let normed = rms_norm(&x, &layer.input_norm, self.cfg.rms_norm_eps);
                let head_dim = layer.head_dim(&self.cfg);
                let kv_heads = layer.kv_heads(&self.cfg);
                let shared_kv_source = self.cfg.shared_kv_source_layer(layer_idx);
                let (mut q, kv_projection) = if shared_kv_source.is_none() {
                    let (q, k, v) =
                        self.matvec3(&layer.q_proj, &layer.k_proj, &layer.v_proj, &normed);
                    (q, Some((k, v)))
                } else {
                    (self.matvec(&layer.q_proj, &normed), None)
                };
                apply_lora_if_present(
                    &mut q,
                    &normed,
                    lora.and_then(|layer| layer.q_proj.as_ref()),
                );

                for head in 0..self.cfg.num_attention_heads {
                    let range = head * head_dim..(head + 1) * head_dim;
                    rms_norm_chunks_in_place(
                        &mut q[range.clone()],
                        head_dim,
                        &layer.q_norm,
                        self.cfg.rms_norm_eps,
                    );
                    apply_rope_split_half_cached(
                        &mut q[range],
                        position,
                        &self.rope_tables[layer_idx],
                    );
                }

                if let Some((mut k, mut v)) = kv_projection {
                    apply_lora_if_present(
                        &mut k,
                        &normed,
                        lora.and_then(|layer| layer.k_proj.as_ref()),
                    );
                    apply_lora_if_present(
                        &mut v,
                        &normed,
                        lora.and_then(|layer| layer.v_proj.as_ref()),
                    );
                    for head in 0..kv_heads {
                        let range = head * head_dim..(head + 1) * head_dim;
                        rms_norm_chunks_in_place(
                            &mut k[range.clone()],
                            head_dim,
                            &layer.k_norm,
                            self.cfg.rms_norm_eps,
                        );
                        rms_norm_unit_chunks_in_place(
                            &mut v[range.clone()],
                            head_dim,
                            self.cfg.rms_norm_eps,
                        );
                        apply_rope_split_half_cached(
                            &mut k[range.clone()],
                            position,
                            &self.rope_tables[layer_idx],
                        );
                        cache.set_kv(layer_idx, position, head, &k[range.clone()], &v[range]);
                    }
                }

                let attn = self.attention(
                    AttentionPlan {
                        kv_layer_idx: shared_kv_source.unwrap_or(layer_idx),
                        query_layer_idx: layer_idx,
                        head_dim,
                        kv_heads,
                    },
                    position,
                    &q,
                    cache,
                );
                let mut o_projected = self.matvec(&layer.o_proj, &attn);
                apply_lora_if_present(
                    &mut o_projected,
                    &attn,
                    lora.and_then(|layer| layer.o_proj.as_ref()),
                );
                rms_norm_in_place(
                    &mut o_projected,
                    &layer.post_attention_norm,
                    self.cfg.rms_norm_eps,
                );
                for j in 0..x.len() {
                    x[j] += o_projected[j];
                }

                let normed_mlp = rms_norm(&x, &layer.pre_feedforward_norm, self.cfg.rms_norm_eps);
                let mut down = if let Some(gpu_down) = self.gpu_mlp(layer, &normed_mlp) {
                    gpu_down
                } else {
                    let (mut hidden, up) =
                        self.matvec2(&layer.gate_proj, &layer.up_proj, &normed_mlp);
                    apply_activation_product_in_place(
                        &self.cfg.hidden_activation,
                        &mut hidden,
                        &up,
                    );
                    self.matvec(&layer.down_proj, &hidden)
                };
                rms_norm_in_place(
                    &mut down,
                    &layer.post_feedforward_norm,
                    self.cfg.rms_norm_eps,
                );
                for j in 0..x.len() {
                    x[j] += down[j];
                }

                if let (
                    Some(per_layer_input),
                    Some(per_layer_input_gate),
                    Some(per_layer_projection),
                    Some(post_per_layer_input_norm),
                ) = (
                    per_layer_inputs_batch[i]
                        .as_ref()
                        .and_then(|inputs| inputs.get(layer_idx)),
                    layer.per_layer_input_gate.as_ref(),
                    layer.per_layer_projection.as_ref(),
                    layer.post_per_layer_input_norm.as_ref(),
                ) {
                    let mut gated = self.matvec(per_layer_input_gate, &x);
                    apply_activation_product_in_place(
                        &self.cfg.hidden_activation,
                        &mut gated,
                        per_layer_input,
                    );
                    let mut projected = self.matvec(per_layer_projection, &gated);
                    rms_norm_in_place(
                        &mut projected,
                        post_per_layer_input_norm,
                        self.cfg.rms_norm_eps,
                    );
                    for j in 0..x.len() {
                        x[j] += projected[j];
                    }
                }

                if layer.layer_scalar != 1.0 {
                    for value in &mut x {
                        *value *= layer.layer_scalar;
                    }
                }

                xs[i] = x;
            }
        }

        xs.into_iter()
            .map(|x| {
                let x = rms_norm(&x, &self.final_norm, self.cfg.rms_norm_eps);
                let mut logits = self.matvec(&self.lm_head, &x);
                if let Some(softcap) = self.cfg.final_logit_softcapping {
                    softcap_in_place(&mut logits, softcap);
                }
                logits
            })
            .collect()
    }

    pub fn generate_greedy(&self, prompt: &[usize], new_tokens: usize) -> Vec<usize> {
        assert!(!prompt.is_empty());
        let mut rng = StdRng::seed_from_u64(0);
        self.generate_sampled(prompt, new_tokens, SamplingConfig::default(), &mut rng)
    }

    pub fn generate_greedy_with_lora(
        &self,
        prompt: &[usize],
        new_tokens: usize,
        active_lora: Option<&LoraAdapters>,
    ) -> Vec<usize> {
        assert!(!prompt.is_empty());
        let mut rng = StdRng::seed_from_u64(0);
        self.generate_sampled_with_lora(
            prompt,
            new_tokens,
            SamplingConfig::default(),
            &mut rng,
            active_lora,
        )
    }

    pub fn generate_sampled<R: Rng + ?Sized>(
        &self,
        prompt: &[usize],
        new_tokens: usize,
        sampling: SamplingConfig,
        rng: &mut R,
    ) -> Vec<usize> {
        self.generate_sampled_with_lora(prompt, new_tokens, sampling, rng, self.lora.as_ref())
    }

    pub fn generate_sampled_with_lora<R: Rng + ?Sized>(
        &self,
        prompt: &[usize],
        new_tokens: usize,
        sampling: SamplingConfig,
        rng: &mut R,
        active_lora: Option<&LoraAdapters>,
    ) -> Vec<usize> {
        assert!(!prompt.is_empty());
        let mut cache = self.new_cache_with_capacity(prompt.len() + new_tokens + 1);
        let mut out = prompt.to_vec();
        let mut logits = Vec::new();
        for (pos, token_id) in prompt.iter().copied().enumerate() {
            logits = self.forward_token_with_lora(token_id, pos, &mut cache, active_lora);
        }
        for _ in 0..new_tokens {
            let next = sample_next(&logits, sampling, rng);
            out.push(next);
            let pos = out.len() - 1;
            logits = self.forward_token_with_lora(next, pos, &mut cache, active_lora);
        }
        out
    }

    fn attention(
        &self,
        plan: AttentionPlan,
        position: usize,
        q: &[f32],
        cache: &AnyKvCache,
    ) -> Vec<f32> {
        let mut out = vec![0.0; q.len()];
        let scale = 1.0;
        let group = self.cfg.num_attention_heads / plan.kv_heads;
        let start_position = self
            .cfg
            .layer_sliding_window(plan.query_layer_idx)
            .map(|window| (position + 1).saturating_sub(window))
            .unwrap_or(0);

        let use_sparsity = attention_sparsity_enabled();

        for q_head in 0..self.cfg.num_attention_heads {
            let kv_head = q_head / group;
            let q_start = q_head * plan.head_dim;
            let q_vec = &q[q_start..q_start + plan.head_dim];
            let mut scores = Vec::with_capacity(position + 1 - start_position);

            if use_sparsity && position - start_position > 32 {
                let mut downsampled_scores = Vec::with_capacity(position + 1 - start_position);
                let mut max_ds = f32::MIN;
                for t in start_position..=position {
                    let k_vec = cache.get_key(plan.kv_layer_idx, t, kv_head);
                    let mut ds_sum = 0.0;
                    let mut i = 0;
                    while i < plan.head_dim {
                        ds_sum += q_vec[i] * k_vec[i];
                        i += 4;
                    }
                    downsampled_scores.push(ds_sum);
                    if ds_sum > max_ds {
                        max_ds = ds_sum;
                    }
                }

                let ds_threshold = 1.5;
                for (offset, t) in (start_position..=position).enumerate() {
                    let ds_val = downsampled_scores[offset];
                    if ds_val >= max_ds - ds_threshold || t == position {
                        let k_vec = cache.get_key(plan.kv_layer_idx, t, kv_head);
                        scores.push(crate::ops::dot(q_vec, k_vec) * scale);
                    } else {
                        scores.push(-1e9);
                    }
                }
            } else {
                for t in start_position..=position {
                    let k_vec = cache.get_key(plan.kv_layer_idx, t, kv_head);
                    scores.push(crate::ops::dot(q_vec, k_vec) * scale);
                }
            }

            softmax_in_place(&mut scores);
            let out_head = &mut out[q_start..q_start + plan.head_dim];
            for (offset, prob) in scores.iter().enumerate() {
                let t = start_position + offset;
                let v_vec = cache.get_value(plan.kv_layer_idx, t, kv_head);
                for i in 0..plan.head_dim {
                    out_head[i] += prob * v_vec[i];
                }
            }
        }
        out
    }

    fn per_layer_inputs_for_token(
        &self,
        token_id: usize,
        input_embedding: &[f32],
    ) -> Option<PerLayerInputs> {
        let per_layer_dim = self.cfg.hidden_size_per_layer_input?;
        if let Ok(cache) = self.per_layer_input_cache.lock()
            && let Some(cached) = cache.get(&token_id)
        {
            return Some(Arc::clone(cached));
        }
        let mut token_component = self.token_embedding_per_layer.as_ref().map(|embedding| {
            embedding
                .row(token_id)
                .chunks_exact(per_layer_dim)
                .map(|chunk| {
                    chunk
                        .iter()
                        .map(|value| value * self.cfg.per_layer_embedding_scale)
                        .collect::<Vec<_>>()
                })
                .collect::<Vec<_>>()
        });
        let projection_component = match (
            self.per_layer_model_projection.as_ref(),
            self.per_layer_projection_norm.as_ref(),
        ) {
            (Some(projection), Some(norm)) => {
                let mut projected = projection.matvec(input_embedding);
                let scale = (self.cfg.hidden_size as f32).powf(-0.5);
                for value in &mut projected {
                    *value *= scale;
                }
                Some(
                    projected
                        .chunks_exact(per_layer_dim)
                        .map(|chunk| rms_norm(chunk, norm, self.cfg.rms_norm_eps))
                        .collect::<Vec<_>>(),
                )
            }
            _ => None,
        };

        let result = match (token_component.take(), projection_component) {
            (Some(mut token_inputs), Some(projected_inputs)) => {
                let scale = 2.0_f32.powf(-0.5);
                for (token_layer, projected_layer) in token_inputs.iter_mut().zip(projected_inputs)
                {
                    for (token_value, projected_value) in
                        token_layer.iter_mut().zip(projected_layer)
                    {
                        *token_value = (*token_value + projected_value) * scale;
                    }
                }
                Some(token_inputs)
            }
            (Some(token_inputs), None) => Some(token_inputs),
            (None, Some(projected_inputs)) => Some(projected_inputs),
            (None, None) => None,
        };
        let result = result.map(Arc::new);
        if let Some(inputs) = &result
            && let Ok(mut cache) = self.per_layer_input_cache.lock()
            && cache.len() < PER_LAYER_INPUT_CACHE_LIMIT
        {
            cache.entry(token_id).or_insert_with(|| Arc::clone(inputs));
        }
        result
    }

    fn per_layer_inputs_for_embedding(&self, input_embedding: &[f32]) -> Option<PerLayerInputs> {
        let per_layer_dim = self.cfg.hidden_size_per_layer_input?;
        match (
            self.per_layer_model_projection.as_ref(),
            self.per_layer_projection_norm.as_ref(),
        ) {
            (Some(projection), Some(norm)) => {
                let mut projected = projection.matvec(input_embedding);
                let scale = (self.cfg.hidden_size as f32).powf(-0.5);
                for value in &mut projected {
                    *value *= scale;
                }
                Some(Arc::new(
                    projected
                        .chunks_exact(per_layer_dim)
                        .map(|chunk| rms_norm(chunk, norm, self.cfg.rms_norm_eps))
                        .collect::<Vec<_>>(),
                ))
            }
            _ => None,
        }
    }
}

fn required_role<'a>(roles: &'a HashMap<&str, &str>, role: &str) -> Result<&'a str> {
    roles
        .get(role)
        .copied()
        .with_context(|| format!("missing required tensor role: {role}"))
}

fn apply_activation_product_in_place(name: &str, values: &mut [f32], inputs: &[f32]) {
    assert_eq!(values.len(), inputs.len());
    match name {
        "gelu_pytorch_tanh" | "gelu_fast" | "gelu_approx_tanh" => {
            for (value, input) in values.iter_mut().zip(inputs) {
                *value = gelu_pytorch_tanh(*value) * *input;
            }
        }
        _ => {
            crate::ops::silu_product_in_place(values, inputs);
        }
    }
}

fn load_vec(
    reader: &mut crate::weights::TensorReader,
    tensor_name: &str,
    expected_len: usize,
    role: &str,
) -> Result<Vec<f32>> {
    let (shape, data) = reader
        .read_f32(tensor_name)
        .with_context(|| format!("loading tensor role={role} name={tensor_name}"))?;
    if shape != [expected_len] {
        bail!(
            "shape mismatch for role={role} name={tensor_name}: expected [{expected_len}] got {:?}",
            shape
        );
    }
    Ok(data)
}

fn load_optional_vec(
    reader: &mut crate::weights::TensorReader,
    tensor_name: Option<&str>,
    expected_len: usize,
    role: &str,
) -> Result<Option<Vec<f32>>> {
    tensor_name
        .map(|name| load_vec(reader, name, expected_len, role))
        .transpose()
}

fn load_norm(
    reader: &mut crate::weights::TensorReader,
    tensor_name: &str,
    expected_len: usize,
    role: &str,
    fold_rms_norm: bool,
) -> Result<Vec<f32>> {
    let mut v = load_vec(reader, tensor_name, expected_len, role)?;
    if fold_rms_norm {
        for x in &mut v {
            *x += 1.0;
        }
    }
    Ok(v)
}

fn load_optional_norm(
    reader: &mut crate::weights::TensorReader,
    tensor_name: Option<&str>,
    expected_len: usize,
    role: &str,
    fold_rms_norm: bool,
) -> Result<Option<Vec<f32>>> {
    let mut opt_v = load_optional_vec(reader, tensor_name, expected_len, role)?;
    if fold_rms_norm {
        for x in opt_v.iter_mut().flatten() {
            *x += 1.0;
        }
    }
    Ok(opt_v)
}

fn load_matrix(
    reader: &mut crate::weights::TensorReader,
    tensor_name: &str,
    rows: usize,
    cols: usize,
    role: &str,
) -> Result<Matrix> {
    let (shape, data) = reader
        .read_f32(tensor_name)
        .with_context(|| format!("loading tensor role={role} name={tensor_name}"))?;
    if shape != [rows, cols] {
        bail!(
            "shape mismatch for role={role} name={tensor_name}: expected [{rows}, {cols}] got {:?}",
            shape
        );
    }
    Ok(Matrix::from_row_major(rows, cols, data))
}

fn load_matrix_with_cols(
    reader: &mut crate::weights::TensorReader,
    tensor_name: &str,
    cols: usize,
    role: &str,
) -> Result<Matrix> {
    let (shape, data) = reader
        .read_f32(tensor_name)
        .with_context(|| format!("loading tensor role={role} name={tensor_name}"))?;
    if shape.len() != 2 || shape[1] != cols {
        bail!(
            "shape mismatch for role={role} name={tensor_name}: expected [*, {cols}] got {:?}",
            shape
        );
    }
    Ok(Matrix::from_row_major(shape[0], shape[1], data))
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum QuantMode {
    Q8,
    Q5,
    Q4,
    Q3,
    Q1_58,
}

impl QuantMode {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Q8 => "q8",
            Self::Q5 => "q5",
            Self::Q4 => "q4",
            Self::Q3 => "q3",
            Self::Q1_58 => "q1_58",
        }
    }
}

pub(crate) fn load_quant_matrix(
    reader: &mut crate::weights::TensorReader,
    tensor_name: &str,
    rows: usize,
    cols: usize,
    role: &str,
    mode: QuantMode,
) -> Result<QuantMatrix> {
    let u_name = format!("{}.U_q", tensor_name);
    let v_name = format!("{}.V_q", tensor_name);
    let su_name = format!("{}.scale_u", tensor_name);
    let sv_name = format!("{}.scale_v", tensor_name);

    if reader.index().has(&u_name) && reader.index().has(&v_name) {
        let shard_u = reader.index().shard_for(&u_name).unwrap().to_path_buf();
        let shard_v = reader.index().shard_for(&v_name).unwrap().to_path_buf();

        let mmap_u = reader.get_shard_bytes(&shard_u)?;
        let mmap_v = reader.get_shard_bytes(&shard_v)?;

        let st_u = safetensors::SafeTensors::deserialize(&mmap_u[..])?;
        let st_v = safetensors::SafeTensors::deserialize(&mmap_v[..])?;

        let tv_u = st_u.tensor(&u_name)?;
        let tv_v = st_v.tensor(&v_name)?;

        let u_shape = tv_u.shape().to_vec();
        let v_shape = tv_v.shape().to_vec();

        if u_shape.len() != 2 || v_shape.len() != 2 {
            bail!(
                "invalid SVD tensor dimensions for name={tensor_name}: U_q shape={u_shape:?}, V_q shape={v_shape:?}"
            );
        }

        let m = u_shape[0];
        let r = u_shape[1];
        let n = v_shape[0];
        if r != v_shape[1] {
            bail!(
                "SVD rank mismatch between U_q ({r}) and V_q ({}) for name={tensor_name}",
                v_shape[1]
            );
        }
        if m != rows || n != cols {
            bail!(
                "shape mismatch for SVD role={role} name={tensor_name}: expected [{rows}, {cols}], got [{m}, {n}]"
            );
        }

        let (_, su_val) = reader.read_f32_scalar(&su_name)?;
        let (_, sv_val) = reader.read_f32_scalar(&sv_name)?;

        let mmap_start_u = mmap_u.as_ptr() as usize;
        let data_start_u = tv_u.data().as_ptr() as usize;
        let u_offset = data_start_u - mmap_start_u;

        let mmap_start_v = mmap_v.as_ptr() as usize;
        let data_start_v = tv_v.data().as_ptr() as usize;
        let v_offset = data_start_v - mmap_start_v;

        return Ok(QuantMatrix::Q8Svd(crate::quant::SvdQ8Matrix {
            scale_u: su_val,
            scale_v: sv_val,
            rows: m,
            cols: n,
            rank: r,
            u_mmap: mmap_u,
            u_offset,
            v_mmap: mmap_v,
            v_offset,
        }));
    }

    let tensor = LazyRowTensor::from_reader(reader, tensor_name)
        .with_context(|| format!("mapping tensor role={role} name={tensor_name}"))?;
    if tensor.rows() != rows || tensor.cols() != cols {
        bail!(
            "shape mismatch for role={role} name={tensor_name}: expected [{rows}, {cols}] got [{}, {}]",
            tensor.rows(),
            tensor.cols()
        );
    }
    match mode {
        QuantMode::Q8 => RowQ8Matrix::quantize_lazy_rows(&tensor)
            .map(QuantMatrix::Q8Resident)
            .with_context(|| format!("quantizing tensor role={role} name={tensor_name} to Q8")),
        QuantMode::Q5 => RowQ5Matrix::quantize_lazy_rows(&tensor)
            .map(QuantMatrix::Q5Resident)
            .with_context(|| format!("quantizing tensor role={role} name={tensor_name} to Q5")),
        QuantMode::Q4 => RowQ4Matrix::quantize_lazy_rows(&tensor)
            .map(QuantMatrix::Q4Resident)
            .with_context(|| format!("quantizing tensor role={role} name={tensor_name} to Q4")),
        QuantMode::Q3 | QuantMode::Q1_58 => crate::quant::RowQ3Matrix::quantize_lazy_rows(&tensor)
            .map(QuantMatrix::Q3Resident)
            .with_context(|| format!("quantizing tensor role={role} name={tensor_name} to Q3")),
    }
}

fn load_prequantized_tensor(
    source: ModelSource<'_>,
    tensor_name: &str,
    mode: QuantMode,
) -> Result<Option<QuantMatrix>> {
    let ext = match mode {
        QuantMode::Q8 => "zq8",
        QuantMode::Q5 => "zq5",
        QuantMode::Q4 => "zq4",
        QuantMode::Q3 | QuantMode::Q1_58 => "zq3",
    };
    let file_name = format!("{}.{}", tensor_name, ext);

    match source {
        ModelSource::Dir(model_dir) => {
            let path = model_dir.join(&file_name);
            if path.exists() {
                let mmap = crate::mmap_utils::map_read_only(&path)?;
                let storage = crate::weights::ByteStorage::Mmap(Arc::new(mmap));
                let matrix = match mode {
                    QuantMode::Q8 => QuantMatrix::Q8Mmap(
                        crate::quant::MmapQ8Matrix::read_zq8_storage(storage, &file_name)?,
                    ),
                    QuantMode::Q5 => QuantMatrix::Q5Mmap(
                        crate::quant::MmapQ5Matrix::read_zq5_storage(storage, &file_name)?,
                    ),
                    QuantMode::Q4 => QuantMatrix::Q4Mmap(
                        crate::quant::MmapQ4Matrix::read_zq4_storage(storage, &file_name)?,
                    ),
                    QuantMode::Q3 | QuantMode::Q1_58 => QuantMatrix::Q3Mmap(
                        crate::quant::MmapQ3Matrix::read_zq3_storage(storage, &file_name)?,
                    ),
                };
                Ok(Some(matrix))
            } else {
                Ok(None)
            }
        }
        ModelSource::InMemory { files, .. } => {
            if let Some(arc_bytes) = files.get(&file_name) {
                let storage = crate::weights::ByteStorage::Memory(arc_bytes.clone());
                let matrix = match mode {
                    QuantMode::Q8 => QuantMatrix::Q8Mmap(
                        crate::quant::MmapQ8Matrix::read_zq8_storage(storage, &file_name)?,
                    ),
                    QuantMode::Q5 => QuantMatrix::Q5Mmap(
                        crate::quant::MmapQ5Matrix::read_zq5_storage(storage, &file_name)?,
                    ),
                    QuantMode::Q4 => QuantMatrix::Q4Mmap(
                        crate::quant::MmapQ4Matrix::read_zq4_storage(storage, &file_name)?,
                    ),
                    QuantMode::Q3 | QuantMode::Q1_58 => QuantMatrix::Q3Mmap(
                        crate::quant::MmapQ3Matrix::read_zq3_storage(storage, &file_name)?,
                    ),
                };
                Ok(Some(matrix))
            } else {
                Ok(None)
            }
        }
    }
}

#[allow(clippy::too_many_arguments)]
fn load_quant_matrix_cached(
    source: ModelSource<'_>,
    reader: &mut crate::weights::TensorReader,
    tensor_name: &str,
    rows: usize,
    cols: usize,
    role: &str,
    mode: QuantMode,
    cache_dir: Option<&Path>,
) -> Result<QuantMatrix> {
    if let Some(matrix) = load_prequantized_tensor(source, tensor_name, mode)? {
        if matrix.rows() != rows || matrix.cols() != cols {
            bail!(
                "shape mismatch for prequantized tensor role={role} name={tensor_name}: expected [{rows}, {cols}] got [{}, {}]",
                matrix.rows(),
                matrix.cols()
            );
        }
        return Ok(matrix);
    }

    let u_name = format!("{}.U_q", tensor_name);
    let v_name = format!("{}.V_q", tensor_name);
    let is_svd = reader.index().has(&u_name) && reader.index().has(&v_name);

    if let (false, Some(dir)) = (is_svd, cache_dir) {
        let path = quant_cache_path(dir, tensor_name, mode);
        if let Some(matrix) = read_quant_cache_if_valid(&path, rows, cols, mode)? {
            return Ok(matrix);
        }
    }

    let matrix = load_quant_matrix(reader, tensor_name, rows, cols, role, mode)?;
    if let Some(dir) = cache_dir {
        let path = quant_cache_path(dir, tensor_name, mode);
        write_quant_cache(&path, &matrix, mode)?;
    }
    Ok(matrix)
}

fn load_quant_matrix_with_cols(
    reader: &mut crate::weights::TensorReader,
    tensor_name: &str,
    cols: usize,
    role: &str,
    mode: QuantMode,
) -> Result<QuantMatrix> {
    let u_name = format!("{}.U_q", tensor_name);
    let v_name = format!("{}.V_q", tensor_name);
    let su_name = format!("{}.scale_u", tensor_name);
    let sv_name = format!("{}.scale_v", tensor_name);

    if reader.index().has(&u_name) && reader.index().has(&v_name) {
        let shard_u = reader.index().shard_for(&u_name).unwrap().to_path_buf();
        let shard_v = reader.index().shard_for(&v_name).unwrap().to_path_buf();

        let mmap_u = reader.get_shard_bytes(&shard_u)?;
        let mmap_v = reader.get_shard_bytes(&shard_v)?;

        let st_u = safetensors::SafeTensors::deserialize(&mmap_u[..])?;
        let st_v = safetensors::SafeTensors::deserialize(&mmap_v[..])?;

        let tv_u = st_u.tensor(&u_name)?;
        let tv_v = st_v.tensor(&v_name)?;

        let u_shape = tv_u.shape().to_vec();
        let v_shape = tv_v.shape().to_vec();

        if u_shape.len() != 2 || v_shape.len() != 2 {
            bail!(
                "invalid SVD tensor dimensions for name={tensor_name}: U_q shape={u_shape:?}, V_q shape={v_shape:?}"
            );
        }

        let m = u_shape[0];
        let r = u_shape[1];
        let n = v_shape[0];
        if r != v_shape[1] {
            bail!(
                "SVD rank mismatch between U_q ({r}) and V_q ({}) for name={tensor_name}",
                v_shape[1]
            );
        }
        if n != cols {
            bail!(
                "shape mismatch for SVD role={role} name={tensor_name}: expected [*, {cols}], got [{m}, {n}]"
            );
        }

        let (_, su_val) = reader.read_f32_scalar(&su_name)?;
        let (_, sv_val) = reader.read_f32_scalar(&sv_name)?;

        let mmap_start_u = mmap_u.as_ptr() as usize;
        let data_start_u = tv_u.data().as_ptr() as usize;
        let u_offset = data_start_u - mmap_start_u;

        let mmap_start_v = mmap_v.as_ptr() as usize;
        let data_start_v = tv_v.data().as_ptr() as usize;
        let v_offset = data_start_v - mmap_start_v;

        return Ok(QuantMatrix::Q8Svd(crate::quant::SvdQ8Matrix {
            scale_u: su_val,
            scale_v: sv_val,
            rows: m,
            cols: n,
            rank: r,
            u_mmap: mmap_u,
            u_offset,
            v_mmap: mmap_v,
            v_offset,
        }));
    }

    let tensor = LazyRowTensor::from_reader(reader, tensor_name)
        .with_context(|| format!("mapping tensor role={role} name={tensor_name}"))?;
    if tensor.cols() != cols {
        bail!(
            "shape mismatch for role={role} name={tensor_name}: expected [*, {cols}] got [{}, {}]",
            tensor.rows(),
            tensor.cols()
        );
    }
    match mode {
        QuantMode::Q8 => RowQ8Matrix::quantize_lazy_rows(&tensor)
            .map(QuantMatrix::Q8Resident)
            .with_context(|| format!("quantizing tensor role={role} name={tensor_name} to Q8")),
        QuantMode::Q5 => RowQ5Matrix::quantize_lazy_rows(&tensor)
            .map(QuantMatrix::Q5Resident)
            .with_context(|| format!("quantizing tensor role={role} name={tensor_name} to Q5")),
        QuantMode::Q4 => RowQ4Matrix::quantize_lazy_rows(&tensor)
            .map(QuantMatrix::Q4Resident)
            .with_context(|| format!("quantizing tensor role={role} name={tensor_name} to Q4")),
        QuantMode::Q3 | QuantMode::Q1_58 => crate::quant::RowQ3Matrix::quantize_lazy_rows(&tensor)
            .map(QuantMatrix::Q3Resident)
            .with_context(|| format!("quantizing tensor role={role} name={tensor_name} to Q3")),
    }
}

#[allow(clippy::too_many_arguments)]
fn load_quant_matrix_with_cols_cached(
    source: ModelSource<'_>,
    reader: &mut crate::weights::TensorReader,
    tensor_name: &str,
    cols: usize,
    role: &str,
    mode: QuantMode,
    cache_dir: Option<&Path>,
) -> Result<QuantMatrix> {
    if let Some(matrix) = load_prequantized_tensor(source, tensor_name, mode)? {
        if matrix.cols() != cols {
            bail!(
                "shape mismatch for prequantized tensor role={role} name={tensor_name}: expected [*, {cols}] got [{}, {}]",
                matrix.rows(),
                matrix.cols()
            );
        }
        return Ok(matrix);
    }

    let u_name = format!("{}.U_q", tensor_name);
    let v_name = format!("{}.V_q", tensor_name);
    let is_svd = reader.index().has(&u_name) && reader.index().has(&v_name);

    if let (false, Some(dir)) = (is_svd, cache_dir) {
        let path = quant_cache_path(dir, tensor_name, mode);
        if let Some(matrix) = read_quant_cache_with_cols_if_valid(&path, cols, mode)? {
            return Ok(matrix);
        }
    }

    let matrix = load_quant_matrix_with_cols(reader, tensor_name, cols, role, mode)?;
    if let Some(dir) = cache_dir {
        let path = quant_cache_path(dir, tensor_name, mode);
        write_quant_cache(&path, &matrix, mode)?;
    }
    Ok(matrix)
}

fn read_quant_cache_if_valid(
    path: &Path,
    rows: usize,
    cols: usize,
    mode: QuantMode,
) -> Result<Option<QuantMatrix>> {
    if !path.exists() {
        return Ok(None);
    }
    let matrix = match mode {
        QuantMode::Q8 => QuantMatrix::read_zq8_mmap(path)?,
        QuantMode::Q5 => QuantMatrix::read_zq5_mmap(path)?,
        QuantMode::Q4 => QuantMatrix::read_zq4_mmap(path)?,
        QuantMode::Q3 | QuantMode::Q1_58 => QuantMatrix::read_zq3_mmap(path)?,
    };
    if matrix.rows() == rows && matrix.cols() == cols {
        Ok(Some(matrix))
    } else {
        Ok(None)
    }
}

fn read_quant_cache_with_cols_if_valid(
    path: &Path,
    cols: usize,
    mode: QuantMode,
) -> Result<Option<QuantMatrix>> {
    if !path.exists() {
        return Ok(None);
    }
    let matrix = match mode {
        QuantMode::Q8 => QuantMatrix::read_zq8_mmap(path)?,
        QuantMode::Q5 => QuantMatrix::read_zq5_mmap(path)?,
        QuantMode::Q4 => QuantMatrix::read_zq4_mmap(path)?,
        QuantMode::Q3 | QuantMode::Q1_58 => QuantMatrix::read_zq3_mmap(path)?,
    };
    if matrix.cols() == cols {
        Ok(Some(matrix))
    } else {
        Ok(None)
    }
}

fn write_quant_cache(path: &Path, matrix: &QuantMatrix, mode: QuantMode) -> Result<()> {
    match (mode, matrix) {
        (QuantMode::Q8, QuantMatrix::Q8Resident(resident)) => {
            resident.write_zq8(path)?;
        }
        (QuantMode::Q5, QuantMatrix::Q5Resident(resident)) => {
            resident.write_zq5(path)?;
        }
        (QuantMode::Q4, QuantMatrix::Q4Resident(resident)) => {
            resident.write_zq4(path)?;
        }
        (QuantMode::Q3 | QuantMode::Q1_58, QuantMatrix::Q3Resident(resident)) => {
            resident.write_zq3(path)?;
        }
        _ => {}
    }
    Ok(())
}

fn write_manifest_for_cache(cache_dir: &Path, mode: QuantMode, cfg: &GemmaConfig) -> Result<()> {
    use serde_json::json;
    let ext = match mode {
        QuantMode::Q8 => "zq8",
        QuantMode::Q5 => "zq5",
        QuantMode::Q4 => "zq4",
        QuantMode::Q3 | QuantMode::Q1_58 => "zq3",
    };
    let mut tensors = Vec::new();
    if cache_dir.exists() {
        for entry in std::fs::read_dir(cache_dir)? {
            let entry = entry?;
            let path = entry.path();
            if path.is_file() && path.extension().and_then(|e| e.to_str()) == Some(ext) {
                let filename = path.file_name().unwrap().to_str().unwrap().to_string();
                tensors.push(filename);
            }
        }
    }
    tensors.sort();
    let manifest = json!({
        "engine_version": "1.0",
        "quant_mode": match mode {
            QuantMode::Q8 => "q8",
            QuantMode::Q5 => "q5",
            QuantMode::Q4 => "q4",
            QuantMode::Q3 => "q3",
            QuantMode::Q1_58 => "q1_58",
        },
        "model_architecture": "gemma",
        "hidden_size": cfg.hidden_size,
        "num_hidden_layers": cfg.num_hidden_layers,
        "num_attention_heads": cfg.num_attention_heads,
        "num_key_value_heads": cfg.num_key_value_heads,
        "vocab_size": cfg.vocab_size,
        "tensor_files": tensors,
    });
    let manifest_path = cache_dir.join("manifest.json");
    let content = serde_json::to_string_pretty(&manifest)?;
    std::fs::write(manifest_path, content)?;
    Ok(())
}

fn quant_cache_path(cache_dir: &Path, tensor_name: &str, mode: QuantMode) -> PathBuf {
    use std::collections::hash_map::DefaultHasher;
    use std::hash::{Hash, Hasher};

    let mut hasher = DefaultHasher::new();
    tensor_name.hash(&mut hasher);
    let hash = hasher.finish();
    let sanitized: String = tensor_name
        .chars()
        .map(|ch| {
            if ch.is_ascii_alphanumeric() || ch == '-' || ch == '_' {
                ch
            } else {
                '_'
            }
        })
        .take(96)
        .collect();
    let ext = match mode {
        QuantMode::Q8 => "zq8",
        QuantMode::Q5 => "zq5",
        QuantMode::Q4 => "zq4",
        QuantMode::Q3 | QuantMode::Q1_58 => "zq3",
    };
    cache_dir.join(format!("{hash:016x}_{sanitized}.{ext}"))
}

#[cfg(test)]
mod tests {
    use super::*;
    use safetensors::{Dtype, View, serialize_to_file};
    use std::borrow::Cow;
    use std::fs;
    use std::path::Path;
    use tempfile::TempDir;

    #[test]
    fn tiny_model_generates_deterministically() {
        let model_a = NativeGemma::seeded_tiny(7);
        let model_b = NativeGemma::seeded_tiny(7);
        let a = model_a.generate_greedy(&[1, 2, 3], 8);
        let b = model_b.generate_greedy(&[1, 2, 3], 8);
        assert_eq!(a, b);
        assert_eq!(a.len(), 11);
    }

    #[test]
    fn per_layer_token_inputs_are_cached_by_token_id() {
        let mut model = NativeGemma::seeded_tiny(7);
        let per_layer_dim = 2;
        let per_token_cols = model.layers.len() * per_layer_dim;
        model.cfg.hidden_size_per_layer_input = Some(per_layer_dim);
        model.token_embedding_per_layer = Some(RowMatrix::Dense(Matrix::from_row_major(
            model.cfg.vocab_size,
            per_token_cols,
            (0..model.cfg.vocab_size * per_token_cols)
                .map(|idx| idx as f32 * 0.01)
                .collect(),
        )));

        let input = model.token_embedding.row(3).to_vec();
        assert_eq!(model.per_layer_input_cache.lock().unwrap().len(), 0);
        let first = model.per_layer_inputs_for_token(3, &input).unwrap();
        assert_eq!(model.per_layer_input_cache.lock().unwrap().len(), 1);
        let second = model.per_layer_inputs_for_token(3, &input).unwrap();

        assert_eq!(first, second);
        assert_eq!(model.per_layer_input_cache.lock().unwrap().len(), 1);
    }

    #[test]
    fn forward_logits_match_vocab() {
        let model = NativeGemma::seeded_tiny(42);
        let mut cache = model.new_cache();
        let logits = model.forward_token(3, 0, &mut cache);
        assert_eq!(logits.len(), model.cfg.vocab_size);
        assert!(logits.iter().all(|v| v.is_finite()));
    }

    #[test]
    fn cuneiform_concept_enters_attention_without_token_id() {
        let model = NativeGemma::seeded_tiny(11);
        let concept = crate::cuneiform::Concept6D::new(1, 2, 3, 4, 5, 6);

        let mut native_cache = model.new_cache_with_capacity(1);
        let native_logits = model.forward_cuneiform_concept(concept, 0, &mut native_cache);
        assert_eq!(native_logits.len(), model.cfg.vocab_size);
        assert!(native_logits.iter().all(|value| value.is_finite()));

        let mut native_output_cache = model.new_cache_with_capacity(1);
        let native_output =
            model.forward_cuneiform_concepts_output(&[concept], 0, &mut native_output_cache);
        assert_eq!(native_output.logits, native_logits);
        assert_eq!(native_output.hidden_state.len(), model.cfg.hidden_size);
        assert!(
            native_output
                .hidden_state
                .iter()
                .all(|value| value.is_finite())
        );

        let mut token_cache = model.new_cache_with_capacity(1);
        let token_logits = model.forward_token(
            concept.vocab_projector_id(model.cfg.vocab_size),
            0,
            &mut token_cache,
        );
        assert_ne!(native_logits, token_logits);

        let quantized = QuantizedGemma::from_native(&model);
        let mut quant_concept_cache = quantized.new_cache_with_capacity(1);
        let quant_concept_logits =
            quantized.forward_cuneiform_concept(concept, 0, &mut quant_concept_cache);
        assert_eq!(quant_concept_logits.len(), quantized.cfg.vocab_size);
        assert!(quant_concept_logits.iter().all(|value| value.is_finite()));

        let mut quant_output_cache = quantized.new_cache_with_capacity(1);
        let quant_output =
            quantized.forward_cuneiform_concepts_output(&[concept], 0, &mut quant_output_cache);
        assert_eq!(quant_output.logits, quant_concept_logits);
        assert_eq!(quant_output.hidden_state.len(), quantized.cfg.hidden_size);
        assert!(
            quant_output
                .hidden_state
                .iter()
                .all(|value| value.is_finite())
        );

        let mut quant_token_cache = quantized.new_cache_with_capacity(1);
        let quant_token_logits = quantized.forward_token(
            concept.vocab_projector_id(quantized.cfg.vocab_size),
            0,
            &mut quant_token_cache,
        );
        assert_ne!(quant_concept_logits, quant_token_logits);
    }

    #[test]
    fn paged_batch_cache_keeps_sequence_ids_isolated() {
        let model = NativeGemma::seeded_tiny(45);
        let layer_shapes: Vec<_> = model
            .layers
            .iter()
            .map(|layer| (layer.kv_heads(&model.cfg), layer.head_dim(&model.cfg)))
            .collect();
        let mut paged = crate::paged_kv::PagedKvCache::new_with_shapes(&layer_shapes, 4);
        paged.create_sequence(10);
        paged.create_sequence(20);
        let ptr = SharedPagedKvCache(&mut paged as *mut _);
        let mut paged_caches = vec![
            AnyKvCache::Paged {
                cache: ptr,
                sequence_id: 10,
            },
            AnyKvCache::Paged {
                cache: ptr,
                sequence_id: 20,
            },
        ];
        let got = model.forward_batch(&[(3, 0), (4, 0)], &mut paged_caches);

        let mut dense_a = model.new_cache_with_capacity(1);
        let mut dense_b = model.new_cache_with_capacity(1);
        let expected_a = model.forward_token(3, 0, &mut dense_a);
        let expected_b = model.forward_token(4, 0, &mut dense_b);
        assert_eq!(got, vec![expected_a, expected_b]);
        assert_eq!(paged.stats(10).unwrap().token_len, 1);
        assert_eq!(paged.stats(20).unwrap().token_len, 1);
    }

    #[test]
    fn lora_adapter_changes_attention_projection_logits() {
        let base = NativeGemma::seeded_tiny(46);
        let v_rows = base.layers[0].v_proj.rows;
        let hidden = base.cfg.hidden_size;
        let mut a = vec![0.0; hidden];
        a[0] = 0.5;
        let layer = LayerLoraAdapters {
            v_proj: Some(LoraProjection {
                a: Matrix::from_row_major(1, hidden, a),
                b: Matrix::from_row_major(v_rows, 1, vec![0.02; v_rows]),
                alpha: 4.0,
            }),
            ..Default::default()
        };
        let adapted = base.clone().with_lora_adapters(LoraAdapters {
            layers: vec![layer],
        });
        let mut base_cache = base.new_cache_with_capacity(1);
        let mut adapted_cache = adapted.new_cache_with_capacity(1);
        let base_logits = base.forward_token(3, 0, &mut base_cache);
        let adapted_logits = adapted.forward_token(3, 0, &mut adapted_cache);
        assert_ne!(base_logits, adapted_logits);
        assert!(adapted_logits.iter().all(|v| v.is_finite()));
    }

    #[test]
    fn lora_hot_swap_runs_distinct_adapters_without_cloning_base() {
        fn adapter(base: &NativeGemma, value: f32) -> LoraAdapters {
            let v_rows = base.layers[0].v_proj.rows;
            let hidden = base.cfg.hidden_size;
            let mut a = vec![0.0; hidden];
            a[0] = value;
            let layer = LayerLoraAdapters {
                v_proj: Some(LoraProjection {
                    a: Matrix::from_row_major(1, hidden, a),
                    b: Matrix::from_row_major(v_rows, 1, vec![value; v_rows]),
                    alpha: 8.0,
                }),
                ..Default::default()
            };
            LoraAdapters {
                layers: vec![layer],
            }
        }

        let base = NativeGemma::seeded_tiny(47);
        let adapter_a = adapter(&base, 0.05);
        let adapter_b = adapter(&base, -0.07);

        let mut base_cache = base.new_cache_with_capacity(1);
        let mut a_cache = base.new_cache_with_capacity(1);
        let mut b_cache = base.new_cache_with_capacity(1);
        let base_logits = base.forward_token(3, 0, &mut base_cache);
        let a_logits = base.forward_token_with_lora(3, 0, &mut a_cache, Some(&adapter_a));
        let b_logits = base.forward_token_with_lora(3, 0, &mut b_cache, Some(&adapter_b));
        assert_ne!(base_logits, a_logits);
        assert_ne!(a_logits, b_logits);

        let quantized = QuantizedGemma::from_native_with_mode(&base, QuantMode::Q8);
        let mut qa_cache = quantized.new_cache_with_capacity(1);
        let q_logits = quantized.forward_token_with_lora(3, 0, &mut qa_cache, Some(&adapter_a));
        assert_eq!(q_logits.len(), quantized.cfg.vocab_size);
        assert!(q_logits.iter().all(|value| value.is_finite()));
    }

    #[test]
    fn loads_peft_style_lora_adapter_safetensors_and_runs_inference() -> Result<()> {
        let base = NativeGemma::seeded_tiny(48);
        let temp = TempDir::new()?;
        fs::write(
            temp.path().join("adapter_config.json"),
            serde_json::to_vec_pretty(&serde_json::json!({
                "r": 1,
                "lora_alpha": 6.0,
                "target_modules": ["v_proj"]
            }))?,
        )?;

        let hidden = base.cfg.hidden_size;
        let rows = base.layers[0].v_proj.rows;
        let mut a = vec![0.0; hidden];
        a[0] = 0.08;
        let b = vec![0.04; rows];
        let tensors = vec![
            (
                "base_model.model.model.layers.0.self_attn.v_proj.lora_A.default.weight"
                    .to_string(),
                TestTensor {
                    shape: vec![1, hidden],
                    bytes: f32_bytes(&a),
                },
            ),
            (
                "base_model.model.model.layers.0.self_attn.v_proj.lora_B.default.weight"
                    .to_string(),
                TestTensor {
                    shape: vec![rows, 1],
                    bytes: f32_bytes(&b),
                },
            ),
        ];
        serialize_to_file(
            tensors,
            &None,
            &temp.path().join("adapter_model.safetensors"),
        )?;

        let adapter = base.load_lora_adapters(temp.path())?;
        let mut base_cache = base.new_cache_with_capacity(1);
        let mut adapted_cache = base.new_cache_with_capacity(1);
        let base_logits = base.forward_token(3, 0, &mut base_cache);
        let adapted_logits = base.forward_token_with_lora(3, 0, &mut adapted_cache, Some(&adapter));
        assert_ne!(base_logits, adapted_logits);

        let quantized = QuantizedGemma::from_native_with_mode(&base, QuantMode::Q8);
        let quantized_adapter = quantized.load_lora_adapters(temp.path())?;
        let mut quantized_cache = quantized.new_cache_with_capacity(1);
        let quantized_logits =
            quantized.forward_token_with_lora(3, 0, &mut quantized_cache, Some(&quantized_adapter));
        assert_eq!(quantized_logits.len(), quantized.cfg.vocab_size);
        assert!(quantized_logits.iter().all(|value| value.is_finite()));
        Ok(())
    }

    #[test]
    fn sampled_generation_is_deterministic_with_seed() {
        let model = NativeGemma::seeded_tiny(42);
        let mut rng_a = StdRng::seed_from_u64(9);
        let mut rng_b = StdRng::seed_from_u64(9);
        let sampling = SamplingConfig {
            temperature: 0.7,
            top_k: 4,
            top_p: None,
            min_p: None,
        };
        assert_eq!(
            model.generate_sampled(&[1, 2, 3], 8, sampling, &mut rng_a),
            model.generate_sampled(&[1, 2, 3], 8, sampling, &mut rng_b)
        );
    }

    #[test]
    fn q8_runtime_generates_finite_logits() {
        let model = NativeGemma::seeded_tiny(42);
        let q8 = QuantizedGemmaQ8::from_native(&model);
        let mut cache = q8.new_cache();
        let logits = q8.forward_token(3, 0, &mut cache);
        assert_eq!(logits.len(), model.cfg.vocab_size);
        assert!(logits.iter().all(|v| v.is_finite()));
    }

    #[test]
    fn loads_hf_safetensors_model_and_matches_seeded_runtime() {
        let source = NativeGemma::seeded_tiny(123);
        let temp = TempDir::new().unwrap();
        write_hf_fixture(temp.path(), &source).unwrap();

        let loaded = NativeGemma::from_hf_dir(temp.path()).unwrap();
        assert_eq!(loaded.cfg.vocab_size, source.cfg.vocab_size);
        assert_eq!(loaded.cfg.hidden_size, source.cfg.hidden_size);
        assert_eq!(loaded.layers.len(), source.layers.len());

        let prompt = [1, 2, 3, 4];
        assert_eq!(
            loaded.generate_greedy(&prompt, 6),
            source.generate_greedy(&prompt, 6)
        );

        let mut source_cache = source.new_cache();
        let mut loaded_cache = loaded.new_cache();
        let source_logits = source.forward_token(7, 0, &mut source_cache);
        let loaded_logits = loaded.forward_token(7, 0, &mut loaded_cache);
        assert_eq!(source_logits, loaded_logits);
    }

    #[test]
    fn direct_q8_hf_loader_matches_q8_from_native() {
        let source = NativeGemma::seeded_tiny(321);
        let temp = TempDir::new().unwrap();
        write_hf_fixture(temp.path(), &source).unwrap();

        let q8_from_native = QuantizedGemmaQ8::from_native(&source);
        let q8_direct = QuantizedGemmaQ8::from_hf_dir(temp.path()).unwrap();
        let prompt = [1, 2, 3, 4];
        let mut rng_a = StdRng::seed_from_u64(11);
        let mut rng_b = StdRng::seed_from_u64(11);

        assert_eq!(
            q8_direct.generate_sampled(&prompt, 6, SamplingConfig::default(), &mut rng_a),
            q8_from_native.generate_sampled(&prompt, 6, SamplingConfig::default(), &mut rng_b),
        );

        let mut direct_cache = q8_direct.new_cache_with_capacity(1);
        let mut native_cache = q8_from_native.new_cache_with_capacity(1);
        assert_eq!(
            q8_direct.forward_token(7, 0, &mut direct_cache),
            q8_from_native.forward_token(7, 0, &mut native_cache),
        );
    }

    #[test]
    fn cached_direct_q8_hf_loader_reuses_zq8_files() {
        let source = NativeGemma::seeded_tiny(654);
        let temp = TempDir::new().unwrap();
        let cache = TempDir::new().unwrap();
        write_hf_fixture(temp.path(), &source).unwrap();

        let first = QuantizedGemmaQ8::from_hf_dir_with_cache(temp.path(), cache.path()).unwrap();
        let second = QuantizedGemmaQ8::from_hf_dir_with_cache(temp.path(), cache.path()).unwrap();
        let cache_files = fs::read_dir(cache.path()).unwrap().count();

        assert!(cache_files > 0);
        let manifest_path = cache.path().join("manifest.json");
        assert!(manifest_path.exists());
        let manifest_str = fs::read_to_string(&manifest_path).unwrap();
        assert!(manifest_str.contains("quant_mode"));
        assert!(manifest_str.contains("q8"));

        assert!(matches!(&first.token_embedding, QuantMatrix::Q8Resident(_)));
        assert!(matches!(&second.token_embedding, QuantMatrix::Q8Mmap(_)));
        assert!(
            second
                .layers
                .iter()
                .all(|layer| matches!(&layer.q_proj, QuantMatrix::Q8Mmap(_)))
        );
        let prompt = [1, 2, 3, 4];
        let mut rng_a = StdRng::seed_from_u64(12);
        let mut rng_b = StdRng::seed_from_u64(12);
        assert_eq!(
            first.generate_sampled(&prompt, 6, SamplingConfig::default(), &mut rng_a),
            second.generate_sampled(&prompt, 6, SamplingConfig::default(), &mut rng_b),
        );
    }

    #[test]
    fn direct_q5_hf_loader_matches_q5_from_native() {
        let source = NativeGemma::seeded_tiny(321);
        let temp = TempDir::new().unwrap();
        write_hf_fixture(temp.path(), &source).unwrap();

        let q5_from_native = QuantizedGemma::from_native_with_mode(&source, QuantMode::Q5);
        let q5_direct = QuantizedGemma::from_hf_dir_with_mode(temp.path(), QuantMode::Q5).unwrap();
        let prompt = [1, 2, 3, 4];
        let mut rng_a = StdRng::seed_from_u64(11);
        let mut rng_b = StdRng::seed_from_u64(11);

        assert_eq!(
            q5_direct.generate_sampled(&prompt, 6, SamplingConfig::default(), &mut rng_a),
            q5_from_native.generate_sampled(&prompt, 6, SamplingConfig::default(), &mut rng_b),
        );

        let mut direct_cache = q5_direct.new_cache_with_capacity(1);
        let mut native_cache = q5_from_native.new_cache_with_capacity(1);
        assert_eq!(
            q5_direct.forward_token(7, 0, &mut direct_cache),
            q5_from_native.forward_token(7, 0, &mut native_cache),
        );
    }

    #[test]
    fn cached_direct_q5_hf_loader_reuses_zq5_files() {
        let source = NativeGemma::seeded_tiny(654);
        let temp = TempDir::new().unwrap();
        let cache = TempDir::new().unwrap();
        write_hf_fixture(temp.path(), &source).unwrap();

        let first = QuantizedGemma::from_hf_dir_with_cache_and_mode(
            temp.path(),
            cache.path(),
            QuantMode::Q5,
        )
        .unwrap();
        let second = QuantizedGemma::from_hf_dir_with_cache_and_mode(
            temp.path(),
            cache.path(),
            QuantMode::Q5,
        )
        .unwrap();
        let cache_files = fs::read_dir(cache.path()).unwrap().count();

        assert!(cache_files > 0);
        assert!(matches!(&first.token_embedding, QuantMatrix::Q5Resident(_)));
        assert!(matches!(&second.token_embedding, QuantMatrix::Q5Mmap(_)));
        let prompt = [1, 2, 3, 4];
        let mut rng_a = StdRng::seed_from_u64(12);
        let mut rng_b = StdRng::seed_from_u64(12);
        assert_eq!(
            first.generate_sampled(&prompt, 6, SamplingConfig::default(), &mut rng_a),
            second.generate_sampled(&prompt, 6, SamplingConfig::default(), &mut rng_b),
        );
    }

    #[test]
    fn direct_q4_hf_loader_matches_q4_from_native() {
        let source = NativeGemma::seeded_tiny(321);
        let temp = TempDir::new().unwrap();
        write_hf_fixture(temp.path(), &source).unwrap();

        let q4_from_native = QuantizedGemma::from_native_with_mode(&source, QuantMode::Q4);
        let q4_direct = QuantizedGemma::from_hf_dir_with_mode(temp.path(), QuantMode::Q4).unwrap();
        let prompt = [1, 2, 3, 4];
        let mut rng_a = StdRng::seed_from_u64(11);
        let mut rng_b = StdRng::seed_from_u64(11);

        assert_eq!(
            q4_direct.generate_sampled(&prompt, 6, SamplingConfig::default(), &mut rng_a),
            q4_from_native.generate_sampled(&prompt, 6, SamplingConfig::default(), &mut rng_b),
        );

        let mut direct_cache = q4_direct.new_cache_with_capacity(1);
        let mut native_cache = q4_from_native.new_cache_with_capacity(1);
        assert_eq!(
            q4_direct.forward_token(7, 0, &mut direct_cache),
            q4_from_native.forward_token(7, 0, &mut native_cache),
        );
    }

    #[test]
    fn cached_direct_q4_hf_loader_reuses_zq4_files() {
        let source = NativeGemma::seeded_tiny(654);
        let temp = TempDir::new().unwrap();
        let cache = TempDir::new().unwrap();
        write_hf_fixture(temp.path(), &source).unwrap();

        let first = QuantizedGemma::from_hf_dir_with_cache_and_mode(
            temp.path(),
            cache.path(),
            QuantMode::Q4,
        )
        .unwrap();
        let second = QuantizedGemma::from_hf_dir_with_cache_and_mode(
            temp.path(),
            cache.path(),
            QuantMode::Q4,
        )
        .unwrap();
        let cache_files = fs::read_dir(cache.path()).unwrap().count();

        assert!(cache_files > 0);
        assert!(matches!(&first.token_embedding, QuantMatrix::Q4Resident(_)));
        assert!(matches!(&second.token_embedding, QuantMatrix::Q4Mmap(_)));
        let prompt = [1, 2, 3, 4];
        let mut rng_a = StdRng::seed_from_u64(12);
        let mut rng_b = StdRng::seed_from_u64(12);
        assert_eq!(
            first.generate_sampled(&prompt, 6, SamplingConfig::default(), &mut rng_a),
            second.generate_sampled(&prompt, 6, SamplingConfig::default(), &mut rng_b),
        );
    }

    #[derive(Clone)]
    struct TestTensor {
        shape: Vec<usize>,
        bytes: Vec<u8>,
    }

    impl View for TestTensor {
        fn dtype(&self) -> Dtype {
            Dtype::F32
        }

        fn shape(&self) -> &[usize] {
            &self.shape
        }

        fn data(&self) -> Cow<'_, [u8]> {
            Cow::Borrowed(&self.bytes)
        }

        fn data_len(&self) -> usize {
            self.bytes.len()
        }
    }

    fn write_hf_fixture(dir: &Path, model: &NativeGemma) -> Result<()> {
        let cfg = &model.cfg;
        fs::write(
            dir.join("config.json"),
            serde_json::to_vec_pretty(&serde_json::json!({
                "model_type": "gemma",
                "vocab_size": cfg.vocab_size,
                "hidden_size": cfg.hidden_size,
                "intermediate_size": cfg.intermediate_size,
                "num_hidden_layers": cfg.num_hidden_layers,
                "num_attention_heads": cfg.num_attention_heads,
                "num_key_value_heads": cfg.num_key_value_heads,
                "head_dim": cfg.head_dim,
                "rms_norm_eps": cfg.rms_norm_eps,
                "rope_theta": cfg.rope_theta,
                "max_position_embeddings": cfg.max_position_embeddings
            }))?,
        )?;

        let mut tensors = Vec::<(String, TestTensor)>::new();
        push_matrix(
            &mut tensors,
            "model.embed_tokens.weight",
            &model.token_embedding,
        );
        push_vec(&mut tensors, "model.norm.weight", &model.final_norm);
        push_matrix(&mut tensors, "lm_head.weight", &model.lm_head);

        for (idx, layer) in model.layers.iter().enumerate() {
            let prefix = format!("model.layers.{idx}");
            push_vec(
                &mut tensors,
                &format!("{prefix}.input_layernorm.weight"),
                &layer.input_norm,
            );
            push_vec(
                &mut tensors,
                &format!("{prefix}.post_attention_layernorm.weight"),
                &layer.post_attention_norm,
            );
            push_vec(
                &mut tensors,
                &format!("{prefix}.pre_feedforward_layernorm.weight"),
                &layer.pre_feedforward_norm,
            );
            push_vec(
                &mut tensors,
                &format!("{prefix}.post_feedforward_layernorm.weight"),
                &layer.post_feedforward_norm,
            );
            push_vec(
                &mut tensors,
                &format!("{prefix}.self_attn.q_norm.weight"),
                &layer.q_norm,
            );
            push_vec(
                &mut tensors,
                &format!("{prefix}.self_attn.k_norm.weight"),
                &layer.k_norm,
            );
            push_matrix(
                &mut tensors,
                &format!("{prefix}.self_attn.q_proj.weight"),
                &layer.q_proj,
            );
            push_matrix(
                &mut tensors,
                &format!("{prefix}.self_attn.k_proj.weight"),
                &layer.k_proj,
            );
            push_matrix(
                &mut tensors,
                &format!("{prefix}.self_attn.v_proj.weight"),
                &layer.v_proj,
            );
            push_matrix(
                &mut tensors,
                &format!("{prefix}.self_attn.o_proj.weight"),
                &layer.o_proj,
            );
            push_matrix(
                &mut tensors,
                &format!("{prefix}.mlp.gate_proj.weight"),
                &layer.gate_proj,
            );
            push_matrix(
                &mut tensors,
                &format!("{prefix}.mlp.up_proj.weight"),
                &layer.up_proj,
            );
            push_matrix(
                &mut tensors,
                &format!("{prefix}.mlp.down_proj.weight"),
                &layer.down_proj,
            );
            push_vec(
                &mut tensors,
                &format!("{prefix}.layer_scalar"),
                &[layer.layer_scalar],
            );
        }

        serialize_to_file(tensors, &None, &dir.join("model.safetensors"))?;
        Ok(())
    }

    fn push_matrix(tensors: &mut Vec<(String, TestTensor)>, name: &str, matrix: &Matrix) {
        tensors.push((
            name.to_owned(),
            TestTensor {
                shape: vec![matrix.rows, matrix.cols],
                bytes: f32_bytes(&matrix.data),
            },
        ));
    }

    fn push_vec(tensors: &mut Vec<(String, TestTensor)>, name: &str, data: &[f32]) {
        tensors.push((
            name.to_owned(),
            TestTensor {
                shape: vec![data.len()],
                bytes: f32_bytes(data),
            },
        ));
    }

    fn f32_bytes(values: &[f32]) -> Vec<u8> {
        values.iter().flat_map(|v| v.to_le_bytes()).collect()
    }

    #[test]
    fn q8_logits_fixture_matches_expected_prefix() {
        if let Ok(model) =
            QuantizedGemma::from_hf_dir_with_mode(Path::new("tiny-gemma-fixture"), QuantMode::Q8)
        {
            let mut cache = model.new_cache();
            let logits = model.forward_token(2, 0, &mut cache);
            assert_eq!(logits.len(), 1000);
            let best_token = crate::ops::argmax(&logits);
            assert_eq!(best_token, 331);
        }
    }

    #[test]
    fn test_prequantized_capsule_loading_in_memory() -> Result<()> {
        let model = NativeGemma::seeded_e4b_mock(1234);
        let temp_dir = tempfile::tempdir()?;

        let q_head = RowQ5Matrix::quantize(&model.lm_head);
        let quant_matrix = QuantMatrix::Q5Resident(q_head);
        let cache_path = temp_dir.path().join("lm_head.weight.zq5");
        write_quant_cache(&cache_path, &quant_matrix, QuantMode::Q5)?;

        let zq5_bytes = fs::read(&cache_path)?;
        let mut files = HashMap::new();
        let config_json = serde_json::to_vec(&serde_json::json!({
            "vocab_size": 32,
            "hidden_size": 96,
            "intermediate_size": 128,
            "num_hidden_layers": 2,
            "num_attention_heads": 12,
            "num_key_value_heads": 4,
            "head_dim": 8,
            "rms_norm_eps": 1e-6,
            "rope_theta": 10000.0,
            "max_position_embeddings": 512,
            "layer_types": ["full_attention", "full_attention"],
            "hidden_activation": "gelu_pytorch_tanh"
        }))?;

        files.insert("lm_head.weight.zq5".to_string(), Arc::new(zq5_bytes));

        let source = ModelSource::InMemory {
            config_json: &config_json,
            files: &files,
        };

        let mut reader = crate::weights::TensorReader::from_in_memory(files.clone())?;

        let loaded = load_quant_matrix_cached(
            source,
            &mut reader,
            "lm_head.weight",
            model.lm_head.rows,
            model.lm_head.cols,
            "lm_head",
            QuantMode::Q5,
            None,
        )?;

        assert!(matches!(loaded, QuantMatrix::Q5Mmap(_)));
        assert_eq!(loaded.rows(), model.lm_head.rows);
        assert_eq!(loaded.cols(), model.lm_head.cols);

        Ok(())
    }

    #[test]
    fn test_prequantized_capsule_v2_end_to_end() -> Result<()> {
        let model = NativeGemma::seeded_e4b_mock(1234);
        let temp_dir = tempfile::tempdir()?;

        let mut files = HashMap::new();

        let quantize_and_store = |name: &str,
                                  matrix: &Matrix,
                                  files: &mut HashMap<String, Arc<Vec<u8>>>|
         -> Result<()> {
            let q = RowQ5Matrix::quantize(matrix);
            let quant_matrix = QuantMatrix::Q5Resident(q);
            let zq_path = temp_dir.path().join("tmp.zq5");
            write_quant_cache(&zq_path, &quant_matrix, QuantMode::Q5)?;
            let bytes = fs::read(&zq_path)?;
            files.insert(format!("{}.zq5", name), Arc::new(bytes));
            Ok(())
        };

        quantize_and_store(
            "model.embed_tokens.weight",
            &model.token_embedding,
            &mut files,
        )?;
        quantize_and_store("lm_head.weight", &model.lm_head, &mut files)?;

        let mut unquantized_tensors = Vec::new();
        let mut push_vec = |name: &str, data: &[f32]| {
            unquantized_tensors.push((
                name.to_owned(),
                TestTensor {
                    shape: vec![data.len()],
                    bytes: f32_bytes(data),
                },
            ));
        };

        push_vec("model.norm.weight", &model.final_norm);

        for (idx, layer) in model.layers.iter().enumerate() {
            let prefix = format!("model.layers.{idx}");
            push_vec(
                &format!("{prefix}.input_layernorm.weight"),
                &layer.input_norm,
            );
            push_vec(
                &format!("{prefix}.post_attention_layernorm.weight"),
                &layer.post_attention_norm,
            );
            push_vec(
                &format!("{prefix}.pre_feedforward_layernorm.weight"),
                &layer.pre_feedforward_norm,
            );
            push_vec(
                &format!("{prefix}.post_feedforward_layernorm.weight"),
                &layer.post_feedforward_norm,
            );
            push_vec(&format!("{prefix}.self_attn.q_norm.weight"), &layer.q_norm);
            push_vec(&format!("{prefix}.self_attn.k_norm.weight"), &layer.k_norm);
            push_vec(&format!("{prefix}.layer_scalar"), &[layer.layer_scalar]);

            quantize_and_store(
                &format!("{prefix}.self_attn.q_proj.weight"),
                &layer.q_proj,
                &mut files,
            )?;
            quantize_and_store(
                &format!("{prefix}.self_attn.k_proj.weight"),
                &layer.k_proj,
                &mut files,
            )?;
            quantize_and_store(
                &format!("{prefix}.self_attn.v_proj.weight"),
                &layer.v_proj,
                &mut files,
            )?;
            quantize_and_store(
                &format!("{prefix}.self_attn.o_proj.weight"),
                &layer.o_proj,
                &mut files,
            )?;
            quantize_and_store(
                &format!("{prefix}.mlp.gate_proj.weight"),
                &layer.gate_proj,
                &mut files,
            )?;
            quantize_and_store(
                &format!("{prefix}.mlp.up_proj.weight"),
                &layer.up_proj,
                &mut files,
            )?;
            quantize_and_store(
                &format!("{prefix}.mlp.down_proj.weight"),
                &layer.down_proj,
                &mut files,
            )?;
        }

        let safetensors_path = temp_dir.path().join("model.safetensors");
        serialize_to_file(unquantized_tensors, &None, &safetensors_path)?;
        let safetensors_bytes = fs::read(&safetensors_path)?;
        files.insert("model.safetensors".to_string(), Arc::new(safetensors_bytes));

        let config_json = serde_json::to_vec(&serde_json::json!({
            "vocab_size": model.cfg.vocab_size,
            "hidden_size": model.cfg.hidden_size,
            "intermediate_size": model.cfg.intermediate_size,
            "num_hidden_layers": model.cfg.num_hidden_layers,
            "num_attention_heads": model.cfg.num_attention_heads,
            "num_key_value_heads": model.cfg.num_key_value_heads,
            "head_dim": model.cfg.head_dim,
            "rms_norm_eps": model.cfg.rms_norm_eps,
            "rope_theta": model.cfg.rope_theta,
            "max_position_embeddings": model.cfg.max_position_embeddings,
            "layer_types": ["full_attention", "full_attention"],
            "hidden_activation": "gelu_pytorch_tanh"
        }))?;

        let source = ModelSource::InMemory {
            config_json: &config_json,
            files: &files,
        };

        let q_model = QuantizedGemma::from_source_inner(source, None, QuantMode::Q5)?;

        let mut cache = q_model.new_cache();
        let logits = q_model.forward_token(2, 0, &mut cache);
        assert_eq!(logits.len(), model.cfg.vocab_size);
        assert!(logits.iter().all(|v| v.is_finite()));

        Ok(())
    }

    #[test]
    fn test_early_exit_skipping() {
        let stable_hidden = [1.0, 1.0, 1.0, 1.0];
        let varying_hidden = [0.0, 10.0, 0.0, 10.0];

        assert!(should_semantic_early_exit(6, &stable_hidden, 100.0));
        assert!(!should_semantic_early_exit(5, &stable_hidden, 100.0));
        assert!(!should_semantic_early_exit(6, &varying_hidden, 1.0));
    }

    #[test]
    fn test_thermal_pressure_approximate_kernel() {
        let len: usize = 33;
        let scale = 0.25;
        let x: Vec<f32> = (0..len).map(|idx| (idx as f32 - 8.0) / 5.0).collect();

        let q8_row: Vec<i8> = (0..len).map(|idx| (idx % 9) as i8 - 4).collect();
        let q8_expected = q8_row
            .iter()
            .zip(&x)
            .enumerate()
            .filter(|(idx, _)| (idx / 16) % 2 == 0)
            .map(|(_, (q, value))| *q as f32 * *value)
            .sum::<f32>()
            * scale
            * 2.0;
        assert_eq!(
            crate::kernels::q8_i8_dot_f32_scaled_thermal_high(&q8_row, &x, scale),
            q8_expected
        );

        let q4_values: Vec<i8> = (0..len).map(|idx| (idx % 15) as i8 - 7).collect();
        let mut q4_packed = vec![0_u8; len.div_ceil(2)];
        for (idx, q) in q4_values.iter().copied().enumerate() {
            let nibble = (q + 8) as u8 & 0x0f;
            let byte = &mut q4_packed[idx / 2];
            if idx.is_multiple_of(2) {
                *byte = (*byte & 0xf0) | nibble;
            } else {
                *byte = (*byte & 0x0f) | (nibble << 4);
            }
        }
        let q4_expected = q4_values
            .iter()
            .zip(&x)
            .enumerate()
            .filter(|(idx, _)| (idx / 16) % 2 == 0)
            .map(|(_, (q, value))| *q as f32 * *value)
            .sum::<f32>()
            * scale
            * 2.0;
        assert_eq!(
            crate::kernels::q4_dot_f32_scaled_thermal_high(&q4_packed, &x, scale),
            q4_expected
        );

        let q5_values: Vec<i8> = (0..len).map(|idx| (idx % 31) as i8 - 15).collect();
        let mut q5_packed = vec![0_u8; (len * 5).div_ceil(8)];
        for (idx, q) in q5_values.iter().copied().enumerate() {
            let code = (q + 16) as u8 & 0x1f;
            for bit in 0..5 {
                let dst = idx * 5 + bit;
                if ((code >> bit) & 1) == 1 {
                    q5_packed[dst / 8] |= 1 << (dst % 8);
                }
            }
        }
        let q5_expected = q5_values
            .iter()
            .zip(&x)
            .enumerate()
            .filter(|(idx, _)| (idx / 16) % 2 == 0)
            .map(|(_, (q, value))| *q as f32 * *value)
            .sum::<f32>()
            * scale
            * 2.0;
        assert_eq!(
            crate::kernels::q5_dot_f32_scaled_thermal_high(&q5_packed, 0, &x, scale),
            q5_expected
        );
    }

    #[test]
    fn q4_forward_default_mode_keeps_logits_finite() {
        let base = NativeGemma::seeded_tiny(78);
        let q_model = QuantizedGemma::from_native_with_mode(&base, QuantMode::Q4);
        let mut cache = q_model.new_cache();
        let logits = q_model.forward_token(2, 0, &mut cache);
        assert_eq!(logits.len(), q_model.cfg.vocab_size);
        assert!(logits.iter().all(|value| value.is_finite()));
    }

    #[test]
    fn test_speculative_entropy_calculation() {
        let logits = vec![10.0, 1.0, 1.0, 1.0];
        let entropy = crate::sampling::calculate_entropy(&logits);
        assert!(entropy < 0.5, "Entropy is low: {entropy}");

        let flat_logits = vec![1.0, 1.0, 1.0, 1.0];
        let flat_entropy = crate::sampling::calculate_entropy(&flat_logits);
        assert!(flat_entropy > 1.0, "Entropy is high: {flat_entropy}");
    }

    #[test]
    fn dynamic_q8_activation_forward_tracks_q8_activation_f32_path() {
        let native = NativeGemma::seeded_tiny(2026);
        let q8 = QuantizedGemma::from_native_with_mode(&native, QuantMode::Q8);
        let q8i = QuantizedGemma::from_native_with_mode(&native, QuantMode::Q8)
            .with_activation_mode(QuantizedActivationMode::DynamicInt8);

        let mut q8_cache = q8.new_cache();
        let mut q8i_cache = q8i.new_cache();
        let reference = q8.forward_token(2, 0, &mut q8_cache);
        let candidate = q8i.forward_token(2, 0, &mut q8i_cache);

        assert_eq!(candidate.len(), reference.len());
        assert!(candidate.iter().all(|value| value.is_finite()));
        assert!(crate::quant::relative_l2_error(&reference, &candidate) < 0.05);
    }

    #[test]
    fn e4b_mock_model_runs_and_generates_deterministically() {
        let model_a = NativeGemma::seeded_e4b_mock(1234);
        let model_b = NativeGemma::seeded_e4b_mock(1234);
        let a = model_a.generate_greedy(&[1, 2, 3], 4);
        let b = model_b.generate_greedy(&[1, 2, 3], 4);
        assert_eq!(a, b);
        assert_eq!(a.len(), 7);

        let q8 = QuantizedGemmaQ8::from_native(&model_a);
        let mut cache = q8.new_cache();
        let logits = q8.forward_token(3, 0, &mut cache);
        assert_eq!(logits.len(), model_a.cfg.vocab_size);
        assert!(logits.iter().all(|v| v.is_finite()));
    }
}
