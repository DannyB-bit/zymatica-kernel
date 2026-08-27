use crate::model::QuantMode;
use crate::quant::{RowQ3Matrix, RowQ4Matrix, RowQ5Matrix, RowQ8Matrix};
use crate::tensor::Matrix;
use anyhow::{Context, Result, bail};
use half::f16;
use std::collections::HashMap;
use std::collections::hash_map::DefaultHasher;
use std::fs::File;
use std::hash::{Hash, Hasher};
use std::io::{Read, Seek, SeekFrom};
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

#[derive(Debug, Clone)]
pub struct GgufTensorInfo {
    pub name: String,
    pub dimensions: Vec<usize>,
    pub ggml_type: u32,
    pub offset: u64,
}

pub struct GgufReader {
    pub tensors: HashMap<String, GgufTensorInfo>,
    file: File,
    data_payload_start: u64,
}

impl GgufReader {
    pub fn open(path: impl AsRef<Path>) -> Result<Self> {
        let mut file = File::open(path)?;

        let mut magic = [0_u8; 4];
        file.read_exact(&mut magic)?;
        if &magic != b"GGUF" {
            bail!("Not a valid GGUF file");
        }

        let mut version_bytes = [0_u8; 4];
        file.read_exact(&mut version_bytes)?;
        let version = u32::from_le_bytes(version_bytes);
        if version != 2 && version != 3 {
            bail!("Unsupported GGUF version: {}", version);
        }

        let mut tensor_count_bytes = [0_u8; 8];
        file.read_exact(&mut tensor_count_bytes)?;
        let tensor_count = u64::from_le_bytes(tensor_count_bytes);

        let mut kv_count_bytes = [0_u8; 8];
        file.read_exact(&mut kv_count_bytes)?;
        let kv_count = u64::from_le_bytes(kv_count_bytes);

        let mut alignment = 32u64;

        for _ in 0..kv_count {
            let key = read_gguf_string(&mut file)?;
            let mut val_type_bytes = [0_u8; 4];
            file.read_exact(&mut val_type_bytes)?;
            let val_type = u32::from_le_bytes(val_type_bytes);

            let val_size = skip_gguf_value(&mut file, val_type)?;
            if key == "general.alignment" && val_type == 4 {
                let curr = file.stream_position()?;
                file.seek(SeekFrom::Current(-(val_size as i64)))?;
                let mut align_bytes = [0_u8; 4];
                file.read_exact(&mut align_bytes)?;
                alignment = u32::from_le_bytes(align_bytes) as u64;
                file.seek(SeekFrom::Start(curr))?;
            }
        }

        let mut tensors = HashMap::new();
        for _ in 0..tensor_count {
            let name = read_gguf_string(&mut file)?;

            let mut num_dims_bytes = [0_u8; 4];
            file.read_exact(&mut num_dims_bytes)?;
            let num_dims = u32::from_le_bytes(num_dims_bytes) as usize;

            let mut dimensions = Vec::with_capacity(num_dims);
            for _ in 0..num_dims {
                let mut dim_bytes = [0_u8; 8];
                file.read_exact(&mut dim_bytes)?;
                dimensions.push(u64::from_le_bytes(dim_bytes) as usize);
            }

            let mut type_bytes = [0_u8; 4];
            file.read_exact(&mut type_bytes)?;
            let ggml_type = u32::from_le_bytes(type_bytes);

            let mut offset_bytes = [0_u8; 8];
            file.read_exact(&mut offset_bytes)?;
            let offset = u64::from_le_bytes(offset_bytes);

            tensors.insert(
                name.clone(),
                GgufTensorInfo {
                    name,
                    dimensions,
                    ggml_type,
                    offset,
                },
            );
        }

        let pos = file.stream_position()?;
        let data_payload_start = (pos + alignment - 1) & !(alignment - 1);

        Ok(Self {
            tensors,
            file,
            data_payload_start,
        })
    }

    pub fn read_tensor_f32(&mut self, name: &str) -> Result<Vec<f32>> {
        let info = self.tensors.get(name).context("tensor not found")?.clone();

        let mut total_elements: usize = 1;
        for &dim in &info.dimensions {
            total_elements = total_elements
                .checked_mul(dim)
                .context("dimension overflow")?;
        }
        if total_elements > 500_000_000 {
            bail!("GGUF tensor too large: {} elements", total_elements);
        }

        let abs_offset = self
            .data_payload_start
            .checked_add(info.offset)
            .context("abs_offset overflow")?;

        let file_len = self.file.metadata()?.len();
        let bytes_to_read = match info.ggml_type {
            0 => Some((total_elements as u64).checked_mul(4).unwrap_or(0)),
            1 => Some((total_elements as u64).checked_mul(2).unwrap_or(0)),
            2 => Some(((total_elements / 32) as u64).checked_mul(18).unwrap_or(0)),
            6 => Some(((total_elements / 32) as u64).checked_mul(22).unwrap_or(0)),
            8 => Some(((total_elements / 32) as u64).checked_mul(34).unwrap_or(0)),
            _ => None,
        };
        if let Some(bytes) = bytes_to_read {
            let end_offset = abs_offset
                .checked_add(bytes)
                .context("end_offset overflow")?;
            if end_offset > file_len {
                bail!(
                    "GGUF tensor '{}' bounds exceed file size: expected offset {}..{} but file len is {}",
                    name,
                    abs_offset,
                    end_offset,
                    file_len
                );
            }
        }

        self.file.seek(SeekFrom::Start(abs_offset))?;

        match info.ggml_type {
            0 => {
                let mut data = vec![0.0f32; total_elements];
                let mut buf = vec![0_u8; total_elements * 4];
                self.file.read_exact(&mut buf)?;
                for i in 0..total_elements {
                    let mut b = [0_u8; 4];
                    b.copy_from_slice(&buf[i * 4..i * 4 + 4]);
                    data[i] = f32::from_le_bytes(b);
                }
                Ok(data)
            }
            1 => {
                let mut data = vec![0.0f32; total_elements];
                let mut buf = vec![0_u8; total_elements * 2];
                self.file.read_exact(&mut buf)?;
                for i in 0..total_elements {
                    let mut b = [0_u8; 2];
                    b.copy_from_slice(&buf[i * 2..i * 2 + 2]);
                    data[i] = f16::from_bits(u16::from_le_bytes(b)).to_f32();
                }
                Ok(data)
            }
            2 => {
                let blocks = total_elements / 32;
                let mut data = vec![0.0f32; total_elements];
                let mut buf = vec![0_u8; blocks * 18];
                self.file.read_exact(&mut buf)?;
                for b in 0..blocks {
                    let d_bits = u16::from_le_bytes([buf[b * 18], buf[b * 18 + 1]]);
                    let d = f16::from_bits(d_bits).to_f32();
                    let qs = &buf[b * 18 + 2..b * 18 + 18];
                    for i in 0..16 {
                        let low = qs[i] & 0x0f;
                        let high = qs[i] >> 4;
                        data[b * 32 + i] = (low as i8 - 8) as f32 * d;
                        data[b * 32 + i + 16] = (high as i8 - 8) as f32 * d;
                    }
                }
                Ok(data)
            }
            6 => {
                let blocks = total_elements / 32;
                let mut data = vec![0.0f32; total_elements];
                let mut buf = vec![0_u8; blocks * 22];
                self.file.read_exact(&mut buf)?;
                for b in 0..blocks {
                    let d_bits = u16::from_le_bytes([buf[b * 22], buf[b * 22 + 1]]);
                    let d = f16::from_bits(d_bits).to_f32();
                    let qh = u32::from_le_bytes([
                        buf[b * 22 + 2],
                        buf[b * 22 + 3],
                        buf[b * 22 + 4],
                        buf[b * 22 + 5],
                    ]);
                    let qs = &buf[b * 22 + 6..b * 22 + 22];
                    for i in 0..16 {
                        let low = qs[i] & 0x0f;
                        let high = qs[i] >> 4;
                        let bit_low = (qh >> i) & 1;
                        let bit_high = (qh >> (i + 16)) & 1;

                        let val_low = (low | (bit_low << 4) as u8) as i8 - 16;
                        let val_high = (high | (bit_high << 4) as u8) as i8 - 16;

                        data[b * 32 + i] = val_low as f32 * d;
                        data[b * 32 + i + 16] = val_high as f32 * d;
                    }
                }
                Ok(data)
            }
            8 => {
                let blocks = total_elements / 32;
                let mut data = vec![0.0f32; total_elements];
                let mut buf = vec![0_u8; blocks * 34];
                self.file.read_exact(&mut buf)?;
                for b in 0..blocks {
                    let d_bits = u16::from_le_bytes([buf[b * 34], buf[b * 34 + 1]]);
                    let d = f16::from_bits(d_bits).to_f32();
                    let qs = &buf[b * 34 + 2..b * 34 + 34];
                    for i in 0..32 {
                        let q = qs[i] as i8;
                        data[b * 32 + i] = q as f32 * d;
                    }
                }
                Ok(data)
            }
            t => bail!("Unsupported GGUF GGML type: {}", t),
        }
    }
}

fn read_gguf_string(reader: &mut impl Read) -> Result<String> {
    let mut len_bytes = [0_u8; 8];
    reader.read_exact(&mut len_bytes)?;
    let len = u64::from_le_bytes(len_bytes) as usize;
    if len > 10 * 1024 * 1024 {
        // 10 MB limit for safety
        bail!("GGUF string too long: {} bytes (max 10MB)", len);
    }
    let mut buf = vec![0_u8; len];
    reader.read_exact(&mut buf)?;
    Ok(String::from_utf8(buf)?)
}

fn skip_gguf_value(reader: &mut impl Read, val_type: u32) -> Result<u64> {
    match val_type {
        0 | 1 | 7 => {
            let mut buf = [0_u8; 1];
            reader.read_exact(&mut buf)?;
            Ok(1)
        }
        2 | 3 => {
            let mut buf = [0_u8; 2];
            reader.read_exact(&mut buf)?;
            Ok(2)
        }
        4..=6 => {
            let mut buf = [0_u8; 4];
            reader.read_exact(&mut buf)?;
            Ok(4)
        }
        8 => {
            let s = read_gguf_string(reader)?;
            Ok(8 + s.len() as u64)
        }
        9 => {
            let mut array_type_bytes = [0_u8; 4];
            reader.read_exact(&mut array_type_bytes)?;
            let array_type = u32::from_le_bytes(array_type_bytes);

            let mut array_len_bytes = [0_u8; 8];
            reader.read_exact(&mut array_len_bytes)?;
            let array_len = u64::from_le_bytes(array_len_bytes);
            if array_len > 1_000_000 {
                bail!("GGUF array too large: {} elements", array_len);
            }

            let mut total_bytes = 12u64;
            for _ in 0..array_len {
                total_bytes += skip_gguf_value(reader, array_type)?;
            }
            Ok(total_bytes)
        }
        t => bail!("Unknown KV value type: {}", t),
    }
}

fn push_conversion(
    reader: &GgufReader,
    tensor_conversions: &mut Vec<(String, String, usize, usize)>,
    roles_key: &str,
    candidates: &[String],
    rows: usize,
    cols: usize,
    required: bool,
) {
    for candidate in candidates {
        if reader.tensors.contains_key(candidate) {
            tensor_conversions.push((roles_key.to_string(), candidate.clone(), rows, cols));
            return;
        }
    }
    if required {
        eprintln!(
            "Warning: could not find required GGUF tensor for role '{}'",
            roles_key
        );
    }
}

pub fn convert_gguf_to_zymatica(
    gguf_path: &Path,
    model_dir: &Path,
    cache_dir: &Path,
    mode: QuantMode,
) -> Result<()> {
    let resolution = crate::gemma_hf::resolve_gemma_dir(model_dir)?;
    let cfg = resolution.config;

    let mut reader = GgufReader::open(gguf_path)?;

    std::fs::create_dir_all(cache_dir)?;

    let mut tensor_conversions = Vec::new();

    push_conversion(
        &reader,
        &mut tensor_conversions,
        "token_embedding",
        &[
            "token_embd.weight".to_string(),
            "model.embed_tokens.weight".to_string(),
        ],
        cfg.vocab_size,
        cfg.hidden_size,
        true,
    );

    push_conversion(
        &reader,
        &mut tensor_conversions,
        "token_embedding_per_layer",
        &["model.embed_tokens_per_layer.weight".to_string()],
        cfg.vocab_size,
        cfg.hidden_size,
        false,
    );

    push_conversion(
        &reader,
        &mut tensor_conversions,
        "per_layer_model_projection",
        &["model.per_layer_model_projection.weight".to_string()],
        cfg.vocab_size,
        cfg.hidden_size,
        false,
    );

    push_conversion(
        &reader,
        &mut tensor_conversions,
        "final_norm",
        &[
            "output_norm.weight".to_string(),
            "model.norm.weight".to_string(),
        ],
        1,
        cfg.hidden_size,
        true,
    );

    push_conversion(
        &reader,
        &mut tensor_conversions,
        "lm_head",
        &["output.weight".to_string(), "lm_head.weight".to_string()],
        cfg.vocab_size,
        cfg.hidden_size,
        true,
    );

    for i in 0..cfg.num_hidden_layers {
        let kv_heads = cfg.num_key_value_heads;
        let head_dim = cfg.head_dim;
        let num_heads = cfg.num_attention_heads;

        push_conversion(
            &reader,
            &mut tensor_conversions,
            &format!("layers.{i}.input_norm"),
            &[
                format!("blk.{i}.attn_norm.weight"),
                format!("model.layers.{i}.input_layernorm.weight"),
            ],
            1,
            cfg.hidden_size,
            true,
        );

        push_conversion(
            &reader,
            &mut tensor_conversions,
            &format!("layers.{i}.post_attention_norm"),
            &[
                format!("blk.{i}.attn_post_norm.weight"),
                format!("model.layers.{i}.post_attention_layernorm.weight"),
            ],
            1,
            cfg.hidden_size,
            true,
        );

        push_conversion(
            &reader,
            &mut tensor_conversions,
            &format!("layers.{i}.pre_feedforward_norm"),
            &[
                format!("blk.{i}.ffn_norm.weight"),
                format!("model.layers.{i}.pre_feedforward_layernorm.weight"),
            ],
            1,
            cfg.hidden_size,
            true,
        );

        push_conversion(
            &reader,
            &mut tensor_conversions,
            &format!("layers.{i}.post_feedforward_norm"),
            &[
                format!("blk.{i}.ffn_post_norm.weight"),
                format!("model.layers.{i}.post_feedforward_layernorm.weight"),
            ],
            1,
            cfg.hidden_size,
            true,
        );

        push_conversion(
            &reader,
            &mut tensor_conversions,
            &format!("layers.{i}.q_norm"),
            &[
                format!("blk.{i}.attn_q_norm.weight"),
                format!("model.layers.{i}.self_attn.q_norm.weight"),
            ],
            1,
            num_heads * head_dim,
            false,
        );

        push_conversion(
            &reader,
            &mut tensor_conversions,
            &format!("layers.{i}.k_norm"),
            &[
                format!("blk.{i}.attn_k_norm.weight"),
                format!("model.layers.{i}.self_attn.k_norm.weight"),
            ],
            1,
            kv_heads * head_dim,
            false,
        );

        push_conversion(
            &reader,
            &mut tensor_conversions,
            &format!("layers.{i}.q_proj"),
            &[
                format!("blk.{i}.attn_q.weight"),
                format!("model.layers.{i}.self_attn.q_proj.weight"),
            ],
            num_heads * head_dim,
            cfg.hidden_size,
            true,
        );

        push_conversion(
            &reader,
            &mut tensor_conversions,
            &format!("layers.{i}.k_proj"),
            &[
                format!("blk.{i}.attn_k.weight"),
                format!("model.layers.{i}.self_attn.k_proj.weight"),
            ],
            kv_heads * head_dim,
            cfg.hidden_size,
            true,
        );

        push_conversion(
            &reader,
            &mut tensor_conversions,
            &format!("layers.{i}.v_proj"),
            &[
                format!("blk.{i}.attn_v.weight"),
                format!("model.layers.{i}.self_attn.v_proj.weight"),
            ],
            kv_heads * head_dim,
            cfg.hidden_size,
            true,
        );

        push_conversion(
            &reader,
            &mut tensor_conversions,
            &format!("layers.{i}.o_proj"),
            &[
                format!("blk.{i}.attn_output.weight"),
                format!("model.layers.{i}.self_attn.o_proj.weight"),
            ],
            cfg.hidden_size,
            num_heads * head_dim,
            true,
        );

        push_conversion(
            &reader,
            &mut tensor_conversions,
            &format!("layers.{i}.gate_proj"),
            &[
                format!("blk.{i}.ffn_gate.weight"),
                format!("model.layers.{i}.mlp.gate_proj.weight"),
            ],
            cfg.intermediate_size,
            cfg.hidden_size,
            true,
        );

        push_conversion(
            &reader,
            &mut tensor_conversions,
            &format!("layers.{i}.up_proj"),
            &[
                format!("blk.{i}.ffn_up.weight"),
                format!("model.layers.{i}.mlp.up_proj.weight"),
            ],
            cfg.intermediate_size,
            cfg.hidden_size,
            true,
        );

        push_conversion(
            &reader,
            &mut tensor_conversions,
            &format!("layers.{i}.down_proj"),
            &[
                format!("blk.{i}.ffn_down.weight"),
                format!("model.layers.{i}.mlp.down_proj.weight"),
            ],
            cfg.hidden_size,
            cfg.intermediate_size,
            true,
        );
    }

    let mut manifest_tensors = Vec::new();

    for (roles_key, gguf_name, rows, cols) in tensor_conversions {
        println!(
            "Converting GGUF tensor '{}' to Zymatica format (shape: [{}, {}])...",
            gguf_name, rows, cols
        );
        let float_data = reader.read_tensor_f32(&gguf_name)?;
        if float_data.len() != rows * cols {
            bail!(
                "Dimension mismatch for tensor '{}': expected {}, got GGUF tensor elements count {}",
                gguf_name,
                rows * cols,
                float_data.len()
            );
        }

        let matrix = Matrix::from_row_major(rows, cols, float_data);

        let cache_path = quant_cache_path(cache_dir, &roles_key, mode);

        match mode {
            QuantMode::Q8 => {
                let q8_matrix = RowQ8Matrix::quantize(&matrix);
                q8_matrix.write_zq8(&cache_path)?;
            }
            QuantMode::Q5 => {
                let q5_matrix = RowQ5Matrix::quantize(&matrix);
                q5_matrix.write_zq5(&cache_path)?;
            }
            QuantMode::Q4 => {
                let q4_matrix = RowQ4Matrix::quantize(&matrix);
                q4_matrix.write_zq4(&cache_path)?;
            }
            QuantMode::Q3 | QuantMode::Q1_58 => {
                let q3_matrix = RowQ3Matrix::quantize(&matrix);
                q3_matrix.write_zq3(&cache_path)?;
            }
        }

        let size_bytes = std::fs::metadata(&cache_path).map(|m| m.len()).unwrap_or(0);
        let filename = cache_path
            .file_name()
            .unwrap()
            .to_str()
            .unwrap()
            .to_string();

        manifest_tensors.push(serde_json::json!({
            "tensor_name": roles_key,
            "filename": filename,
            "size_bytes": size_bytes,
        }));
    }

    manifest_tensors.sort_by(|a, b| {
        a["tensor_name"]
            .as_str()
            .unwrap_or("")
            .cmp(b["tensor_name"].as_str().unwrap_or(""))
    });

    let manifest = serde_json::json!({
        "quant_mode": format!("{:?}", mode).to_lowercase(),
        "vocab_size": cfg.vocab_size,
        "hidden_size": cfg.hidden_size,
        "num_hidden_layers": cfg.num_hidden_layers,
        "tensors": manifest_tensors,
        "created_at": SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs(),
    });

    let manifest_path = cache_dir.join("manifest.json");
    let file = std::fs::File::create(&manifest_path)?;
    serde_json::to_writer_pretty(file, &manifest)?;

    println!(
        "GGUF model converted successfully! Cache manifest written to '{}'.",
        manifest_path.display()
    );
    Ok(())
}

fn quant_cache_path(cache_dir: &Path, tensor_name: &str, mode: QuantMode) -> PathBuf {
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
    cache_dir.join(format!("{}_{:016x}.{}", sanitized, hash, ext))
}
