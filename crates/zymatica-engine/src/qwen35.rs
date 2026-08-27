use crate::model::QuantMode;
use crate::ops::{RopeTrigTable, apply_rope_split_half_cached, dot, silu, softmax_in_place};
use crate::quant::{QuantMatrix, QuantizedActivationMode, RowQ4Matrix, RowQ5Matrix, RowQ8Matrix};
use crate::sampling::{SamplingConfig, sample_next};
use crate::tensor::Tensor3;
use crate::weights::{LazyRowTensor, TensorIndex, TensorReader};
use anyhow::{Context, Result, bail};
use rand::{Rng, SeedableRng, rngs::StdRng};
#[cfg(feature = "parallel")]
use rayon::prelude::*;
use serde::Deserialize;
use serde_json::Value;
use std::collections::hash_map::DefaultHasher;
use std::fs;
use std::hash::{Hash, Hasher};
use std::path::{Path, PathBuf};

#[cfg(feature = "parallel")]
const PARALLEL_QWEN_MATVEC_WORK_ITEMS: usize = 262_144;

#[derive(Debug, Clone, Deserialize)]
struct RawQwen35RootConfig {
    model_type: Option<String>,
    text_config: Option<RawQwen35TextConfig>,
}

#[derive(Debug, Clone, Deserialize)]
struct RawQwen35TextConfig {
    model_type: Option<String>,
    vocab_size: usize,
    hidden_size: usize,
    intermediate_size: usize,
    num_hidden_layers: usize,
    num_attention_heads: usize,
    num_key_value_heads: usize,
    head_dim: usize,
    linear_conv_kernel_dim: usize,
    linear_key_head_dim: usize,
    linear_value_head_dim: usize,
    linear_num_key_heads: usize,
    linear_num_value_heads: usize,
    layer_types: Vec<String>,
    rms_norm_eps: f32,
    max_position_embeddings: usize,
    hidden_act: Option<String>,
    eos_token_id: Option<usize>,
    rope_parameters: Option<Value>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Qwen35Config {
    pub vocab_size: usize,
    pub hidden_size: usize,
    pub intermediate_size: usize,
    pub num_hidden_layers: usize,
    pub num_attention_heads: usize,
    pub num_key_value_heads: usize,
    pub head_dim: usize,
    pub linear_conv_kernel_dim: usize,
    pub linear_key_head_dim: usize,
    pub linear_value_head_dim: usize,
    pub linear_num_key_heads: usize,
    pub linear_num_value_heads: usize,
    pub layer_types: Vec<Qwen35LayerType>,
    pub rms_norm_eps: f32,
    pub max_position_embeddings: usize,
    pub hidden_act: String,
    pub eos_token_id: Option<usize>,
    pub rope_theta: f32,
    pub partial_rotary_factor: f32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Qwen35LayerType {
    LinearAttention,
    FullAttention,
}

#[derive(Debug, Clone)]
pub struct Qwen35TextModel {
    pub cfg: Qwen35Config,
    token_embedding: LinearMatrix,
    layers: Vec<Qwen35Layer>,
    final_norm: Vec<f32>,
    lm_head: LinearMatrix,
    rope_table: RopeTrigTable,
}

#[derive(Debug, Clone)]
struct Qwen35Layer {
    input_norm: Vec<f32>,
    post_attention_norm: Vec<f32>,
    mixer: Qwen35Mixer,
    mlp: Qwen35Mlp,
}

#[derive(Debug, Clone)]
enum Qwen35Mixer {
    Linear(Qwen35LinearAttention),
    Full(Qwen35FullAttention),
}

#[derive(Debug, Clone)]
struct Qwen35Mlp {
    gate_proj: LinearMatrix,
    up_proj: LinearMatrix,
    down_proj: LinearMatrix,
}

#[derive(Debug, Clone)]
struct Qwen35FullAttention {
    q_proj: LinearMatrix,
    k_proj: LinearMatrix,
    v_proj: LinearMatrix,
    o_proj: LinearMatrix,
    q_norm: Vec<f32>,
    k_norm: Vec<f32>,
}

#[derive(Debug, Clone)]
struct Qwen35LinearAttention {
    in_proj_qkv: LinearMatrix,
    in_proj_z: LinearMatrix,
    in_proj_b: LinearMatrix,
    in_proj_a: LinearMatrix,
    conv1d_weight: Vec<f32>,
    dt_bias: Vec<f32>,
    a_log: Vec<f32>,
    norm: Vec<f32>,
    out_proj: LinearMatrix,
}

#[derive(Debug, Clone)]
enum LinearMatrix {
    Lazy(LazyRowTensor),
    Quant(QuantMatrix),
}

#[derive(Debug, Clone)]
pub struct Qwen35Cache {
    layers: Vec<Qwen35LayerCache>,
    max_seq: usize,
}

#[derive(Debug, Clone)]
enum Qwen35LayerCache {
    Linear {
        conv_state: Vec<f32>,
        recurrent_state: Vec<f32>,
    },
    Full {
        keys: Tensor3,
        values: Tensor3,
    },
}

impl Qwen35Config {
    pub fn validate(&self) -> Result<()> {
        if self.layer_types.len() != self.num_hidden_layers {
            bail!(
                "invalid Qwen3.5 config: layer_types len {} != num_hidden_layers {}",
                self.layer_types.len(),
                self.num_hidden_layers
            );
        }
        if !self
            .num_attention_heads
            .is_multiple_of(self.num_key_value_heads)
        {
            bail!(
                "invalid Qwen3.5 config: attention heads {} are not divisible by kv heads {}",
                self.num_attention_heads,
                self.num_key_value_heads
            );
        }
        if !self
            .linear_num_value_heads
            .is_multiple_of(self.linear_num_key_heads)
        {
            bail!(
                "invalid Qwen3.5 config: linear value heads {} are not divisible by linear key heads {}",
                self.linear_num_value_heads,
                self.linear_num_key_heads
            );
        }
        if self.hidden_act != "silu" {
            bail!(
                "unsupported Qwen3.5 activation {}; expected silu",
                self.hidden_act
            );
        }
        let rotary_dim = self.rotary_dim();
        if rotary_dim == 0 || !rotary_dim.is_multiple_of(2) || rotary_dim > self.head_dim {
            bail!(
                "invalid Qwen3.5 rotary dimension {rotary_dim} for head_dim {}",
                self.head_dim
            );
        }
        Ok(())
    }

    fn rotary_dim(&self) -> usize {
        ((self.head_dim as f32) * self.partial_rotary_factor)
            .round()
            .clamp(0.0, self.head_dim as f32) as usize
    }

    fn linear_key_dim(&self) -> usize {
        self.linear_num_key_heads * self.linear_key_head_dim
    }

    fn linear_value_dim(&self) -> usize {
        self.linear_num_value_heads * self.linear_value_head_dim
    }

    fn linear_conv_dim(&self) -> usize {
        self.linear_key_dim() * 2 + self.linear_value_dim()
    }
}

impl RawQwen35RootConfig {
    fn into_config(self) -> Result<Qwen35Config> {
        let root_type = self.model_type.as_deref().unwrap_or_default();
        let raw = self
            .text_config
            .with_context(|| "Qwen3.5 config missing text_config")?;
        let text_type = raw.model_type.as_deref().unwrap_or_default();
        if root_type != "qwen3_5" || text_type != "qwen3_5_text" {
            bail!(
                "not a Qwen3.5 text config: model_type={root_type:?} text_config.model_type={text_type:?}"
            );
        }

        let layer_types = raw
            .layer_types
            .iter()
            .map(|layer_type| match layer_type.as_str() {
                "linear_attention" => Ok(Qwen35LayerType::LinearAttention),
                "full_attention" => Ok(Qwen35LayerType::FullAttention),
                other => bail!("unsupported Qwen3.5 layer type {other}"),
            })
            .collect::<Result<Vec<_>>>()?;

        let rope_theta = raw
            .rope_parameters
            .as_ref()
            .and_then(|value| value.get("rope_theta"))
            .and_then(Value::as_f64)
            .unwrap_or(10_000_000.0) as f32;
        let partial_rotary_factor = raw
            .rope_parameters
            .as_ref()
            .and_then(|value| value.get("partial_rotary_factor"))
            .and_then(Value::as_f64)
            .unwrap_or(0.25) as f32;

        let cfg = Qwen35Config {
            vocab_size: raw.vocab_size,
            hidden_size: raw.hidden_size,
            intermediate_size: raw.intermediate_size,
            num_hidden_layers: raw.num_hidden_layers,
            num_attention_heads: raw.num_attention_heads,
            num_key_value_heads: raw.num_key_value_heads,
            head_dim: raw.head_dim,
            linear_conv_kernel_dim: raw.linear_conv_kernel_dim,
            linear_key_head_dim: raw.linear_key_head_dim,
            linear_value_head_dim: raw.linear_value_head_dim,
            linear_num_key_heads: raw.linear_num_key_heads,
            linear_num_value_heads: raw.linear_num_value_heads,
            layer_types,
            rms_norm_eps: raw.rms_norm_eps,
            max_position_embeddings: raw.max_position_embeddings,
            hidden_act: raw.hidden_act.unwrap_or_else(|| "silu".to_string()),
            eos_token_id: raw.eos_token_id,
            rope_theta,
            partial_rotary_factor,
        };
        cfg.validate()?;
        Ok(cfg)
    }
}

impl Qwen35TextModel {
    pub fn is_qwen35_dir(model_dir: impl AsRef<Path>) -> bool {
        parse_config_file(model_dir.as_ref().join("config.json")).is_ok()
    }

    pub fn parse_config_file(path: impl AsRef<Path>) -> Result<Qwen35Config> {
        parse_config_file(path)
    }

    pub fn from_hf_dir(model_dir: impl AsRef<Path>) -> Result<Self> {
        Self::from_hf_dir_inner(model_dir.as_ref(), None)
    }

    pub fn from_hf_dir_with_mode(model_dir: impl AsRef<Path>, mode: QuantMode) -> Result<Self> {
        Self::from_hf_dir_inner(model_dir.as_ref(), Some((None, mode)))
    }

    pub fn from_hf_dir_with_cache_and_mode(
        model_dir: impl AsRef<Path>,
        cache_dir: impl AsRef<Path>,
        mode: QuantMode,
    ) -> Result<Self> {
        Self::from_hf_dir_inner(model_dir.as_ref(), Some((Some(cache_dir.as_ref()), mode)))
    }

    fn from_hf_dir_inner(
        model_dir: &Path,
        quant: Option<(Option<&Path>, QuantMode)>,
    ) -> Result<Self> {
        let cfg = parse_config_file(model_dir.join("config.json"))
            .with_context(|| format!("parsing Qwen3.5 config in {}", model_dir.display()))?;
        let index = TensorIndex::from_dir(model_dir)
            .with_context(|| format!("indexing Qwen3.5 tensors in {}", model_dir.display()))?;
        let mut reader = TensorReader::from_dir(model_dir)
            .with_context(|| format!("opening Qwen3.5 tensors in {}", model_dir.display()))?;
        Self::from_reader(cfg, &index, &mut reader, quant)
    }

    fn from_reader(
        cfg: Qwen35Config,
        index: &TensorIndex,
        reader: &mut TensorReader,
        quant: Option<(Option<&Path>, QuantMode)>,
    ) -> Result<Self> {
        let token_embedding_name = find_required(
            index,
            &[
                "model.language_model.embed_tokens.weight",
                "language_model.embed_tokens.weight",
                "model.embed_tokens.weight",
            ],
            "token_embedding",
        )?;
        let token_embedding = LinearMatrix::from_reader(
            reader,
            &token_embedding_name,
            cfg.vocab_size,
            cfg.hidden_size,
            "token_embedding",
            quant,
        )?;

        let final_norm = load_qwen_norm(
            reader,
            &find_required(
                index,
                &[
                    "model.language_model.norm.weight",
                    "language_model.norm.weight",
                    "model.norm.weight",
                ],
                "final_norm",
            )?,
            cfg.hidden_size,
            "final_norm",
        )?;

        let lm_head = if let Some(name) = index.find_first([
            "lm_head.weight",
            "model.lm_head.weight",
            "model.language_model.lm_head.weight",
        ]) {
            LinearMatrix::from_reader(
                reader,
                &name,
                cfg.vocab_size,
                cfg.hidden_size,
                "lm_head",
                quant,
            )?
        } else {
            token_embedding.clone()
        };

        let mut layers = Vec::with_capacity(cfg.num_hidden_layers);
        for layer_idx in 0..cfg.num_hidden_layers {
            let fallback_prefixes = [
                format!("model.language_model.layers.{layer_idx}"),
                format!("language_model.layers.{layer_idx}"),
                format!("model.layers.{layer_idx}"),
            ];
            let role_name = |suffix: &str| -> Vec<String> {
                fallback_prefixes
                    .iter()
                    .map(|base| format!("{base}.{suffix}"))
                    .collect()
            };

            let input_norm = load_qwen_norm_from_candidates(
                reader,
                index,
                &role_name("input_layernorm.weight"),
                cfg.hidden_size,
                &format!("layers.{layer_idx}.input_norm"),
            )?;
            let post_attention_norm = load_qwen_norm_from_candidates(
                reader,
                index,
                &role_name("post_attention_layernorm.weight"),
                cfg.hidden_size,
                &format!("layers.{layer_idx}.post_attention_norm"),
            )?;
            let mlp = Qwen35Mlp {
                gate_proj: load_linear_from_candidates(
                    reader,
                    index,
                    &role_name("mlp.gate_proj.weight"),
                    cfg.intermediate_size,
                    cfg.hidden_size,
                    &format!("layers.{layer_idx}.mlp.gate_proj"),
                    quant,
                )?,
                up_proj: load_linear_from_candidates(
                    reader,
                    index,
                    &role_name("mlp.up_proj.weight"),
                    cfg.intermediate_size,
                    cfg.hidden_size,
                    &format!("layers.{layer_idx}.mlp.up_proj"),
                    quant,
                )?,
                down_proj: load_linear_from_candidates(
                    reader,
                    index,
                    &role_name("mlp.down_proj.weight"),
                    cfg.hidden_size,
                    cfg.intermediate_size,
                    &format!("layers.{layer_idx}.mlp.down_proj"),
                    quant,
                )?,
            };

            let mixer = match cfg.layer_types[layer_idx] {
                Qwen35LayerType::LinearAttention => Qwen35Mixer::Linear(Qwen35LinearAttention {
                    in_proj_qkv: load_linear_from_candidates(
                        reader,
                        index,
                        &role_name("linear_attn.in_proj_qkv.weight"),
                        cfg.linear_conv_dim(),
                        cfg.hidden_size,
                        &format!("layers.{layer_idx}.linear_attn.in_proj_qkv"),
                        quant,
                    )?,
                    in_proj_z: load_linear_from_candidates(
                        reader,
                        index,
                        &role_name("linear_attn.in_proj_z.weight"),
                        cfg.linear_value_dim(),
                        cfg.hidden_size,
                        &format!("layers.{layer_idx}.linear_attn.in_proj_z"),
                        quant,
                    )?,
                    in_proj_b: load_linear_from_candidates(
                        reader,
                        index,
                        &role_name("linear_attn.in_proj_b.weight"),
                        cfg.linear_num_value_heads,
                        cfg.hidden_size,
                        &format!("layers.{layer_idx}.linear_attn.in_proj_b"),
                        quant,
                    )?,
                    in_proj_a: load_linear_from_candidates(
                        reader,
                        index,
                        &role_name("linear_attn.in_proj_a.weight"),
                        cfg.linear_num_value_heads,
                        cfg.hidden_size,
                        &format!("layers.{layer_idx}.linear_attn.in_proj_a"),
                        quant,
                    )?,
                    conv1d_weight: load_vec_from_candidates(
                        reader,
                        index,
                        &role_name("linear_attn.conv1d.weight"),
                        cfg.linear_conv_dim() * cfg.linear_conv_kernel_dim,
                        &format!("layers.{layer_idx}.linear_attn.conv1d.weight"),
                    )?,
                    dt_bias: load_vec_from_candidates(
                        reader,
                        index,
                        &role_name("linear_attn.dt_bias"),
                        cfg.linear_num_value_heads,
                        &format!("layers.{layer_idx}.linear_attn.dt_bias"),
                    )?,
                    a_log: load_vec_from_candidates(
                        reader,
                        index,
                        &role_name("linear_attn.A_log"),
                        cfg.linear_num_value_heads,
                        &format!("layers.{layer_idx}.linear_attn.A_log"),
                    )?,
                    norm: load_raw_vec_from_candidates(
                        reader,
                        index,
                        &role_name("linear_attn.norm.weight"),
                        cfg.linear_value_head_dim,
                        &format!("layers.{layer_idx}.linear_attn.norm"),
                    )?,
                    out_proj: load_linear_from_candidates(
                        reader,
                        index,
                        &role_name("linear_attn.out_proj.weight"),
                        cfg.hidden_size,
                        cfg.linear_value_dim(),
                        &format!("layers.{layer_idx}.linear_attn.out_proj"),
                        quant,
                    )?,
                }),
                Qwen35LayerType::FullAttention => Qwen35Mixer::Full(Qwen35FullAttention {
                    q_proj: load_linear_from_candidates(
                        reader,
                        index,
                        &role_name("self_attn.q_proj.weight"),
                        cfg.num_attention_heads * cfg.head_dim * 2,
                        cfg.hidden_size,
                        &format!("layers.{layer_idx}.self_attn.q_proj"),
                        quant,
                    )?,
                    k_proj: load_linear_from_candidates(
                        reader,
                        index,
                        &role_name("self_attn.k_proj.weight"),
                        cfg.num_key_value_heads * cfg.head_dim,
                        cfg.hidden_size,
                        &format!("layers.{layer_idx}.self_attn.k_proj"),
                        quant,
                    )?,
                    v_proj: load_linear_from_candidates(
                        reader,
                        index,
                        &role_name("self_attn.v_proj.weight"),
                        cfg.num_key_value_heads * cfg.head_dim,
                        cfg.hidden_size,
                        &format!("layers.{layer_idx}.self_attn.v_proj"),
                        quant,
                    )?,
                    o_proj: load_linear_from_candidates(
                        reader,
                        index,
                        &role_name("self_attn.o_proj.weight"),
                        cfg.hidden_size,
                        cfg.num_attention_heads * cfg.head_dim,
                        &format!("layers.{layer_idx}.self_attn.o_proj"),
                        quant,
                    )?,
                    q_norm: load_qwen_norm_from_candidates(
                        reader,
                        index,
                        &role_name("self_attn.q_norm.weight"),
                        cfg.head_dim,
                        &format!("layers.{layer_idx}.self_attn.q_norm"),
                    )?,
                    k_norm: load_qwen_norm_from_candidates(
                        reader,
                        index,
                        &role_name("self_attn.k_norm.weight"),
                        cfg.head_dim,
                        &format!("layers.{layer_idx}.self_attn.k_norm"),
                    )?,
                }),
            };

            layers.push(Qwen35Layer {
                input_norm,
                post_attention_norm,
                mixer,
                mlp,
            });
        }

        Ok(Self {
            rope_table: RopeTrigTable::new(
                cfg.max_position_embeddings,
                cfg.rotary_dim(),
                cfg.rope_theta,
                1.0,
            ),
            cfg,
            token_embedding,
            layers,
            final_norm,
            lm_head,
        })
    }

    pub fn new_cache_with_capacity(&self, max_seq: usize) -> Qwen35Cache {
        Qwen35Cache::new(&self.cfg, max_seq)
    }

    pub fn eos_token_id(&self) -> Option<usize> {
        self.cfg.eos_token_id
    }

    pub fn forward_token(
        &self,
        token_id: usize,
        position: usize,
        cache: &mut Qwen35Cache,
    ) -> Vec<f32> {
        let output = self.forward_token_output(token_id, position, cache);
        output.logits
    }

    pub fn forward_token_output(
        &self,
        token_id: usize,
        position: usize,
        cache: &mut Qwen35Cache,
    ) -> crate::model::ForwardOutput {
        assert!(token_id < self.cfg.vocab_size);
        assert!(position < cache.max_seq);
        let mut x = self.token_embedding.row(token_id);

        for (layer_idx, layer) in self.layers.iter().enumerate() {
            let normed = qwen_rms_norm(&x, &layer.input_norm, self.cfg.rms_norm_eps);
            let mixed = match &layer.mixer {
                Qwen35Mixer::Linear(linear) => {
                    self.forward_linear_attention(linear, &normed, &mut cache.layers[layer_idx])
                }
                Qwen35Mixer::Full(full) => self.forward_full_attention(
                    full,
                    &normed,
                    position,
                    &mut cache.layers[layer_idx],
                ),
            };
            add_update_in_place(&mut x, &mixed);

            let normed = qwen_rms_norm(&x, &layer.post_attention_norm, self.cfg.rms_norm_eps);
            let mlp = self.forward_mlp(&layer.mlp, &normed);
            add_update_in_place(&mut x, &mlp);
        }

        let hidden_state = qwen_rms_norm(&x, &self.final_norm, self.cfg.rms_norm_eps);
        let logits = self.lm_head.matvec(&hidden_state);
        crate::model::ForwardOutput {
            logits,
            hidden_state,
        }
    }

    pub fn generate_sampled<R: Rng + ?Sized>(
        &self,
        prompt: &[usize],
        new_tokens: usize,
        sampling: SamplingConfig,
        rng: &mut R,
    ) -> Vec<usize> {
        assert!(!prompt.is_empty());
        let mut cache = self.new_cache_with_capacity(prompt.len() + new_tokens + 1);
        let mut out = prompt.to_vec();
        let mut logits = Vec::new();
        for (pos, token_id) in prompt.iter().copied().enumerate() {
            logits = self.forward_token(token_id, pos, &mut cache);
        }
        for _ in 0..new_tokens {
            let next = sample_next(&logits, sampling, rng);
            out.push(next);
            let pos = out.len() - 1;
            logits = self.forward_token(next, pos, &mut cache);
        }
        out
    }

    pub fn generate_greedy(&self, prompt: &[usize], new_tokens: usize) -> Vec<usize> {
        let mut rng = StdRng::seed_from_u64(0);
        self.generate_sampled(prompt, new_tokens, SamplingConfig::default(), &mut rng)
    }

    fn forward_mlp(&self, mlp: &Qwen35Mlp, x: &[f32]) -> Vec<f32> {
        let (mut gate, up) = LinearMatrix::matvec2(&mlp.gate_proj, &mlp.up_proj, x);
        for (value, up_value) in gate.iter_mut().zip(up) {
            *value = silu(*value) * up_value;
        }
        mlp.down_proj.matvec(&gate)
    }

    fn forward_full_attention(
        &self,
        attention: &Qwen35FullAttention,
        x: &[f32],
        position: usize,
        cache: &mut Qwen35LayerCache,
    ) -> Vec<f32> {
        let Qwen35LayerCache::Full { keys, values } = cache else {
            panic!("Qwen3.5 full attention used with linear cache");
        };
        let (q_gate, mut k, v) =
            LinearMatrix::matvec3(&attention.q_proj, &attention.k_proj, &attention.v_proj, x);
        let mut q = vec![0.0; self.cfg.num_attention_heads * self.cfg.head_dim];
        let mut gate = vec![0.0; self.cfg.num_attention_heads * self.cfg.head_dim];

        for head in 0..self.cfg.num_attention_heads {
            let src = head * self.cfg.head_dim * 2;
            let dst = head * self.cfg.head_dim;
            q[dst..dst + self.cfg.head_dim].copy_from_slice(&q_gate[src..src + self.cfg.head_dim]);
            gate[dst..dst + self.cfg.head_dim]
                .copy_from_slice(&q_gate[src + self.cfg.head_dim..src + self.cfg.head_dim * 2]);
            qwen_rms_norm_in_place(
                &mut q[dst..dst + self.cfg.head_dim],
                &attention.q_norm,
                self.cfg.rms_norm_eps,
            );
            apply_rope_split_half_cached(
                &mut q[dst..dst + self.cfg.rotary_dim()],
                position,
                &self.rope_table,
            );
        }

        for head in 0..self.cfg.num_key_value_heads {
            let range = head * self.cfg.head_dim..(head + 1) * self.cfg.head_dim;
            qwen_rms_norm_in_place(
                &mut k[range.clone()],
                &attention.k_norm,
                self.cfg.rms_norm_eps,
            );
            let rotary_range = range.start..range.start + self.cfg.rotary_dim();
            apply_rope_split_half_cached(&mut k[rotary_range], position, &self.rope_table);
            keys.get_mut(position, head)
                .copy_from_slice(&k[range.clone()]);
            values
                .get_mut(position, head)
                .copy_from_slice(&v[range.clone()]);
        }

        let mut out = vec![0.0; q.len()];
        let group = self.cfg.num_attention_heads / self.cfg.num_key_value_heads;
        let scale = 1.0 / (self.cfg.head_dim as f32).sqrt();
        for q_head in 0..self.cfg.num_attention_heads {
            let kv_head = q_head / group;
            let start = q_head * self.cfg.head_dim;
            let q_vec = &q[start..start + self.cfg.head_dim];
            let mut scores = Vec::with_capacity(position + 1);
            for t in 0..=position {
                scores.push(dot(q_vec, keys.get(t, kv_head)) * scale);
            }
            softmax_in_place(&mut scores);
            let out_head = &mut out[start..start + self.cfg.head_dim];
            for (t, prob) in scores.iter().copied().enumerate() {
                let v_vec = values.get(t, kv_head);
                for i in 0..self.cfg.head_dim {
                    out_head[i] += prob * v_vec[i];
                }
            }
        }

        for (value, gate_value) in out.iter_mut().zip(gate) {
            *value *= sigmoid(gate_value);
        }
        attention.o_proj.matvec(&out)
    }

    fn forward_linear_attention(
        &self,
        attention: &Qwen35LinearAttention,
        x: &[f32],
        cache: &mut Qwen35LayerCache,
    ) -> Vec<f32> {
        let Qwen35LayerCache::Linear {
            conv_state,
            recurrent_state,
        } = cache
        else {
            panic!("Qwen3.5 linear attention used with full-attention cache");
        };

        let mixed = attention.in_proj_qkv.matvec(x);
        let convolved = causal_depthwise_conv_update(
            &mixed,
            conv_state,
            &attention.conv1d_weight,
            self.cfg.linear_conv_kernel_dim,
        );
        let key_dim = self.cfg.linear_key_dim();
        let value_dim = self.cfg.linear_value_dim();
        let query = &convolved[..key_dim];
        let key = &convolved[key_dim..key_dim * 2];
        let value = &convolved[key_dim * 2..key_dim * 2 + value_dim];
        let (z, b, a) = LinearMatrix::matvec3(
            &attention.in_proj_z,
            &attention.in_proj_b,
            &attention.in_proj_a,
            x,
        );

        let repeat = self.cfg.linear_num_value_heads / self.cfg.linear_num_key_heads;
        let mut core = vec![0.0; value_dim];
        for v_head in 0..self.cfg.linear_num_value_heads {
            let k_head = v_head / repeat;
            let q_start = k_head * self.cfg.linear_key_head_dim;
            let v_start = v_head * self.cfg.linear_value_head_dim;
            let q_vec = &query[q_start..q_start + self.cfg.linear_key_head_dim];
            let k_vec = &key[q_start..q_start + self.cfg.linear_key_head_dim];
            let q_norm_scale =
                l2_normalize_scale(q_vec, 1e-6) / (self.cfg.linear_key_head_dim as f32).sqrt();
            let k_norm_scale = l2_normalize_scale(k_vec, 1e-6);

            let beta = sigmoid(b[v_head]);
            let g = (-attention.a_log[v_head].exp()
                * softplus(a[v_head] + attention.dt_bias[v_head]))
            .exp();
            let state_offset =
                v_head * self.cfg.linear_key_head_dim * self.cfg.linear_value_head_dim;
            for state_value in &mut recurrent_state[state_offset
                ..state_offset + self.cfg.linear_key_head_dim * self.cfg.linear_value_head_dim]
            {
                *state_value *= g;
            }

            let mut kv_mem = vec![0.0; self.cfg.linear_value_head_dim];
            for (k_idx, k_value) in k_vec.iter().copied().enumerate() {
                let k_value = k_value * k_norm_scale;
                let row = state_offset + k_idx * self.cfg.linear_value_head_dim;
                for v_idx in 0..self.cfg.linear_value_head_dim {
                    kv_mem[v_idx] += recurrent_state[row + v_idx] * k_value;
                }
            }

            let mut delta = vec![0.0; self.cfg.linear_value_head_dim];
            for v_idx in 0..self.cfg.linear_value_head_dim {
                delta[v_idx] = (value[v_start + v_idx] - kv_mem[v_idx]) * beta;
            }
            for (k_idx, k_value) in k_vec.iter().copied().enumerate() {
                let k_value = k_value * k_norm_scale;
                let row = state_offset + k_idx * self.cfg.linear_value_head_dim;
                for v_idx in 0..self.cfg.linear_value_head_dim {
                    recurrent_state[row + v_idx] += k_value * delta[v_idx];
                }
            }
            for v_idx in 0..self.cfg.linear_value_head_dim {
                let mut sum = 0.0;
                for (k_idx, q_value) in q_vec.iter().copied().enumerate() {
                    let q_value = q_value * q_norm_scale;
                    let row = state_offset + k_idx * self.cfg.linear_value_head_dim;
                    sum += recurrent_state[row + v_idx] * q_value;
                }
                core[v_start + v_idx] = sum;
            }
        }

        for v_head in 0..self.cfg.linear_num_value_heads {
            let start = v_head * self.cfg.linear_value_head_dim;
            qwen_rms_norm_gated_in_place(
                &mut core[start..start + self.cfg.linear_value_head_dim],
                &z[start..start + self.cfg.linear_value_head_dim],
                &attention.norm,
                self.cfg.rms_norm_eps,
            );
        }

        attention.out_proj.matvec(&core)
    }
}

impl Qwen35Cache {
    fn new(cfg: &Qwen35Config, max_seq: usize) -> Self {
        let layers = cfg
            .layer_types
            .iter()
            .map(|layer_type| match layer_type {
                Qwen35LayerType::LinearAttention => Qwen35LayerCache::Linear {
                    conv_state: vec![0.0; cfg.linear_conv_dim() * cfg.linear_conv_kernel_dim],
                    recurrent_state: vec![
                        0.0;
                        cfg.linear_num_value_heads
                            * cfg.linear_key_head_dim
                            * cfg.linear_value_head_dim
                    ],
                },
                Qwen35LayerType::FullAttention => Qwen35LayerCache::Full {
                    keys: Tensor3::zeros(max_seq, cfg.num_key_value_heads, cfg.head_dim),
                    values: Tensor3::zeros(max_seq, cfg.num_key_value_heads, cfg.head_dim),
                },
            })
            .collect();
        Self { layers, max_seq }
    }
}

impl LinearMatrix {
    fn from_reader(
        reader: &mut TensorReader,
        tensor_name: &str,
        rows: usize,
        cols: usize,
        role: &str,
        quant: Option<(Option<&Path>, QuantMode)>,
    ) -> Result<Self> {
        let tensor = LazyRowTensor::from_reader(reader, tensor_name)
            .with_context(|| format!("mapping Qwen3.5 tensor role={role} name={tensor_name}"))?;
        if tensor.rows() != rows || tensor.cols() != cols {
            bail!(
                "shape mismatch for Qwen3.5 role={role} name={tensor_name}: expected [{rows}, {cols}] got [{}, {}]",
                tensor.rows(),
                tensor.cols()
            );
        }
        if let Some((cache_dir, mode)) = quant {
            return quantize_qwen_linear(tensor_name, role, tensor, cache_dir, mode);
        }
        Ok(Self::Lazy(tensor))
    }

    fn cols(&self) -> usize {
        match self {
            Self::Lazy(tensor) => tensor.cols(),
            Self::Quant(matrix) => matrix.cols(),
        }
    }

    fn rows(&self) -> usize {
        match self {
            Self::Lazy(tensor) => tensor.rows(),
            Self::Quant(matrix) => matrix.rows(),
        }
    }

    fn row(&self, row: usize) -> Vec<f32> {
        match self {
            Self::Lazy(tensor) => tensor
                .row_f32(row)
                .unwrap_or_else(|err| panic!("Qwen3.5 lazy row read failed: {err:#}")),
            Self::Quant(matrix) => matrix.dequantize_row(row),
        }
    }

    fn matvec(&self, x: &[f32]) -> Vec<f32> {
        assert_eq!(self.cols(), x.len());
        match self {
            Self::Quant(matrix) => matrix.matvec(x),
            Self::Lazy(tensor) => {
                let mut out = vec![0.0; tensor.rows()];
                #[cfg(feature = "parallel")]
                {
                    if tensor.rows() * tensor.cols() >= PARALLEL_QWEN_MATVEC_WORK_ITEMS {
                        out.par_iter_mut()
                            .enumerate()
                            .for_each(|(row_idx, out_cell)| {
                                *out_cell = tensor.row_dot_f32(row_idx, x).unwrap_or_else(|err| {
                                    panic!("Qwen3.5 lazy row dot failed: {err:#}")
                                });
                            });
                        return out;
                    }
                }
                for (row_idx, out_cell) in out.iter_mut().enumerate() {
                    *out_cell = tensor
                        .row_dot_f32(row_idx, x)
                        .unwrap_or_else(|err| panic!("Qwen3.5 lazy row dot failed: {err:#}"));
                }
                out
            }
        }
    }

    fn matvec2(a: &Self, b: &Self, x: &[f32]) -> (Vec<f32>, Vec<f32>) {
        assert_eq!(a.cols(), x.len());
        assert_eq!(b.cols(), x.len());
        assert_eq!(a.rows(), b.rows());
        match (a, b) {
            (Self::Quant(a), Self::Quant(b)) => {
                QuantMatrix::matvec2_with_activation_mode(a, b, x, QuantizedActivationMode::F32)
            }
            _ => {
                #[cfg(feature = "parallel")]
                {
                    rayon::join(|| a.matvec(x), || b.matvec(x))
                }
                #[cfg(not(feature = "parallel"))]
                {
                    (a.matvec(x), b.matvec(x))
                }
            }
        }
    }

    fn matvec3(a: &Self, b: &Self, c: &Self, x: &[f32]) -> (Vec<f32>, Vec<f32>, Vec<f32>) {
        assert_eq!(a.cols(), x.len());
        assert_eq!(b.cols(), x.len());
        assert_eq!(c.cols(), x.len());
        match (a, b, c) {
            (Self::Quant(a), Self::Quant(b), Self::Quant(c)) => {
                QuantMatrix::matvec3_with_activation_mode(a, b, c, x, QuantizedActivationMode::F32)
            }
            _ => {
                #[cfg(feature = "parallel")]
                {
                    let (out_a, (out_b, out_c)) = rayon::join(
                        || a.matvec(x),
                        || rayon::join(|| b.matvec(x), || c.matvec(x)),
                    );
                    (out_a, out_b, out_c)
                }
                #[cfg(not(feature = "parallel"))]
                {
                    (a.matvec(x), b.matvec(x), c.matvec(x))
                }
            }
        }
    }
}

fn quantize_qwen_linear(
    tensor_name: &str,
    role: &str,
    tensor: LazyRowTensor,
    cache_dir: Option<&Path>,
    mode: QuantMode,
) -> Result<LinearMatrix> {
    if let Some(cache_dir) = cache_dir {
        fs::create_dir_all(cache_dir)
            .with_context(|| format!("creating Qwen quant cache {}", cache_dir.display()))?;
        let path = qwen_quant_cache_path(cache_dir, tensor_name, mode);
        if let Some(cached) =
            read_qwen_quant_cache_if_valid(&path, tensor.rows(), tensor.cols(), mode).with_context(
                || {
                    format!(
                        "reading Qwen quant cache role={role} path={}",
                        path.display()
                    )
                },
            )?
        {
            return Ok(LinearMatrix::Quant(cached));
        }
        let quantized = quantize_qwen_tensor(&tensor, mode)
            .with_context(|| format!("quantizing Qwen tensor role={role} name={tensor_name}"))?;
        write_qwen_quant_cache(&path, &quantized, mode)
            .with_context(|| format!("writing Qwen quant cache {}", path.display()))?;
        write_qwen_manifest_for_cache(cache_dir, mode)?;
        return read_qwen_quant_cache_if_valid(&path, tensor.rows(), tensor.cols(), mode)?
            .map(LinearMatrix::Quant)
            .with_context(|| format!("new Qwen quant cache failed validation {}", path.display()));
    }

    quantize_qwen_tensor(&tensor, mode)
        .map(LinearMatrix::Quant)
        .with_context(|| format!("quantizing Qwen tensor role={role} name={tensor_name}"))
}

fn quantize_qwen_tensor(tensor: &LazyRowTensor, mode: QuantMode) -> Result<QuantMatrix> {
    match mode {
        QuantMode::Q8 => RowQ8Matrix::quantize_lazy_rows(tensor).map(QuantMatrix::Q8Resident),
        QuantMode::Q5 => RowQ5Matrix::quantize_lazy_rows(tensor).map(QuantMatrix::Q5Resident),
        QuantMode::Q4 => RowQ4Matrix::quantize_lazy_rows(tensor).map(QuantMatrix::Q4Resident),
        QuantMode::Q3 | QuantMode::Q1_58 => {
            crate::quant::RowQ3Matrix::quantize_lazy_rows(tensor).map(QuantMatrix::Q3Resident)
        }
    }
}

fn read_qwen_quant_cache_if_valid(
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

fn write_qwen_quant_cache(path: &Path, matrix: &QuantMatrix, mode: QuantMode) -> Result<()> {
    match (mode, matrix) {
        (QuantMode::Q8, QuantMatrix::Q8Resident(resident)) => resident.write_zq8(path)?,
        (QuantMode::Q5, QuantMatrix::Q5Resident(resident)) => resident.write_zq5(path)?,
        (QuantMode::Q4, QuantMatrix::Q4Resident(resident)) => resident.write_zq4(path)?,
        (QuantMode::Q3 | QuantMode::Q1_58, QuantMatrix::Q3Resident(resident)) => {
            resident.write_zq3(path)?
        }
        _ => {}
    }
    Ok(())
}

fn write_qwen_manifest_for_cache(cache_dir: &Path, mode: QuantMode) -> Result<()> {
    let ext = quant_ext(mode);
    let mut tensors = Vec::new();
    for entry in fs::read_dir(cache_dir)? {
        let path = entry?.path();
        if path.is_file() && path.extension().and_then(|value| value.to_str()) == Some(ext) {
            let filename = path
                .file_name()
                .and_then(|value| value.to_str())
                .unwrap_or("")
                .to_string();
            tensors.push(serde_json::json!({
                "filename": filename,
                "size_bytes": fs::metadata(&path)?.len(),
            }));
        }
    }
    tensors.sort_by(|a, b| {
        a["filename"]
            .as_str()
            .unwrap_or("")
            .cmp(b["filename"].as_str().unwrap_or(""))
    });
    let manifest = serde_json::json!({
        "model_type": "qwen3_5",
        "quant_mode": mode.as_str(),
        "tensors": tensors,
        "created_at": std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs(),
    });
    let file = fs::File::create(cache_dir.join("manifest.json"))?;
    serde_json::to_writer_pretty(file, &manifest)?;
    Ok(())
}

fn qwen_quant_cache_path(cache_dir: &Path, tensor_name: &str, mode: QuantMode) -> PathBuf {
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
    cache_dir.join(format!("{hash:016x}_{sanitized}.{}", quant_ext(mode)))
}

fn quant_ext(mode: QuantMode) -> &'static str {
    match mode {
        QuantMode::Q8 => "zq8",
        QuantMode::Q5 => "zq5",
        QuantMode::Q4 => "zq4",
        QuantMode::Q3 | QuantMode::Q1_58 => "zq3",
    }
}

pub fn parse_config_file(path: impl AsRef<Path>) -> Result<Qwen35Config> {
    parse_config_bytes(
        &fs::read(path.as_ref()).with_context(|| format!("reading {}", path.as_ref().display()))?,
    )
}

pub fn parse_config_bytes(bytes: &[u8]) -> Result<Qwen35Config> {
    let raw: RawQwen35RootConfig =
        serde_json::from_slice(bytes).context("parsing Qwen3.5 config JSON")?;
    raw.into_config()
}

pub fn is_qwen35_dir(model_dir: impl AsRef<Path>) -> bool {
    Qwen35TextModel::is_qwen35_dir(model_dir)
}

fn find_required(index: &TensorIndex, candidates: &[&str], role: &str) -> Result<String> {
    index
        .find_first(candidates.iter().copied())
        .with_context(|| format!("missing Qwen3.5 tensor role={role} candidates={candidates:?}"))
}

fn find_required_owned(index: &TensorIndex, candidates: &[String], role: &str) -> Result<String> {
    index
        .find_first(candidates.iter().map(String::as_str))
        .with_context(|| format!("missing Qwen3.5 tensor role={role} candidates={candidates:?}"))
}

fn load_linear_from_candidates(
    reader: &mut TensorReader,
    index: &TensorIndex,
    candidates: &[String],
    rows: usize,
    cols: usize,
    role: &str,
    quant: Option<(Option<&Path>, QuantMode)>,
) -> Result<LinearMatrix> {
    let name = find_required_owned(index, candidates, role)?;
    LinearMatrix::from_reader(reader, &name, rows, cols, role, quant)
}

fn load_raw_vec_from_candidates(
    reader: &mut TensorReader,
    index: &TensorIndex,
    candidates: &[String],
    len: usize,
    role: &str,
) -> Result<Vec<f32>> {
    let name = find_required_owned(index, candidates, role)?;
    load_vec(reader, &name, len, role)
}

fn load_vec_from_candidates(
    reader: &mut TensorReader,
    index: &TensorIndex,
    candidates: &[String],
    len: usize,
    role: &str,
) -> Result<Vec<f32>> {
    load_raw_vec_from_candidates(reader, index, candidates, len, role)
}

fn load_qwen_norm_from_candidates(
    reader: &mut TensorReader,
    index: &TensorIndex,
    candidates: &[String],
    len: usize,
    role: &str,
) -> Result<Vec<f32>> {
    let name = find_required_owned(index, candidates, role)?;
    load_qwen_norm(reader, &name, len, role)
}

fn load_vec(
    reader: &mut TensorReader,
    tensor_name: &str,
    len: usize,
    role: &str,
) -> Result<Vec<f32>> {
    let (shape, data) = reader
        .read_f32(tensor_name)
        .with_context(|| format!("loading Qwen3.5 tensor role={role} name={tensor_name}"))?;
    let actual_len: usize = shape.iter().product();
    if actual_len != len || data.len() != len {
        bail!(
            "shape mismatch for Qwen3.5 role={role} name={tensor_name}: expected {len} values got shape {shape:?}"
        );
    }
    Ok(data)
}

fn load_qwen_norm(
    reader: &mut TensorReader,
    tensor_name: &str,
    len: usize,
    role: &str,
) -> Result<Vec<f32>> {
    let mut data = load_vec(reader, tensor_name, len, role)?;
    for value in &mut data {
        *value += 1.0;
    }
    Ok(data)
}

fn qwen_rms_norm(x: &[f32], weight: &[f32], eps: f32) -> Vec<f32> {
    let mut out = x.to_vec();
    qwen_rms_norm_in_place(&mut out, weight, eps);
    out
}

fn qwen_rms_norm_in_place(values: &mut [f32], weight: &[f32], eps: f32) {
    assert_eq!(values.len(), weight.len());
    let mean_square = values.iter().map(|v| v * v).sum::<f32>() / values.len() as f32;
    let scale = 1.0 / (mean_square + eps).sqrt();
    for (value, weight) in values.iter_mut().zip(weight) {
        *value *= scale * weight;
    }
}

fn qwen_rms_norm_gated_in_place(values: &mut [f32], gate: &[f32], weight: &[f32], eps: f32) {
    assert_eq!(values.len(), gate.len());
    assert_eq!(values.len(), weight.len());
    let mean_square = values.iter().map(|v| v * v).sum::<f32>() / values.len() as f32;
    let scale = 1.0 / (mean_square + eps).sqrt();
    for ((value, gate), weight) in values.iter_mut().zip(gate).zip(weight) {
        *value = *value * scale * weight * silu(*gate);
    }
}

fn l2_normalize_scale(values: &[f32], eps: f32) -> f32 {
    let sum_square = values.iter().map(|value| value * value).sum::<f32>();
    1.0 / (sum_square + eps).sqrt()
}

fn causal_depthwise_conv_update(
    mixed: &[f32],
    conv_state: &mut [f32],
    weights: &[f32],
    kernel: usize,
) -> Vec<f32> {
    assert_eq!(conv_state.len(), mixed.len() * kernel);
    assert_eq!(weights.len(), mixed.len() * kernel);
    let mut out = vec![0.0; mixed.len()];
    for channel in 0..mixed.len() {
        let start = channel * kernel;
        conv_state.copy_within(start + 1..start + kernel, start);
        conv_state[start + kernel - 1] = mixed[channel];
        let sum = dot(
            &conv_state[start..start + kernel],
            &weights[start..start + kernel],
        );
        out[channel] = silu(sum);
    }
    out
}

fn add_update_in_place(out: &mut [f32], update: &[f32]) {
    assert_eq!(out.len(), update.len());
    for (out, update) in out.iter_mut().zip(update) {
        *out += *update;
    }
}

fn sigmoid(x: f32) -> f32 {
    if x >= 0.0 {
        1.0 / (1.0 + (-x).exp())
    } else {
        let exp_x = x.exp();
        exp_x / (1.0 + exp_x)
    }
}

fn softplus(x: f32) -> f32 {
    if x > 20.0 {
        x
    } else if x < -20.0 {
        x.exp()
    } else {
        (1.0 + x.exp()).ln()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use safetensors::Dtype;
    use safetensors::tensor::{View, serialize_to_file};
    use std::borrow::Cow;

    #[derive(Debug)]
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

    #[test]
    fn parses_official_qwen35_small_config_shape() {
        let bytes = br#"{
          "model_type": "qwen3_5",
          "text_config": {
            "model_type": "qwen3_5_text",
            "vocab_size": 248320,
            "hidden_size": 1024,
            "intermediate_size": 3584,
            "num_hidden_layers": 24,
            "num_attention_heads": 8,
            "num_key_value_heads": 2,
            "head_dim": 256,
            "linear_conv_kernel_dim": 4,
            "linear_key_head_dim": 128,
            "linear_value_head_dim": 128,
            "linear_num_key_heads": 16,
            "linear_num_value_heads": 16,
            "layer_types": ["linear_attention", "linear_attention", "linear_attention", "full_attention",
              "linear_attention", "linear_attention", "linear_attention", "full_attention",
              "linear_attention", "linear_attention", "linear_attention", "full_attention",
              "linear_attention", "linear_attention", "linear_attention", "full_attention",
              "linear_attention", "linear_attention", "linear_attention", "full_attention",
              "linear_attention", "linear_attention", "linear_attention", "full_attention"],
            "rms_norm_eps": 1e-6,
            "max_position_embeddings": 262144,
            "hidden_act": "silu",
            "eos_token_id": 248044,
            "rope_parameters": {
              "rope_type": "default",
              "rope_theta": 10000000,
              "partial_rotary_factor": 0.25
            }
          }
        }"#;
        let cfg = parse_config_bytes(bytes).unwrap();
        assert_eq!(cfg.hidden_size, 1024);
        assert_eq!(cfg.num_hidden_layers, 24);
        assert_eq!(cfg.linear_conv_dim(), 6144);
        assert_eq!(cfg.rotary_dim(), 64);
        assert_eq!(cfg.layer_types[3], Qwen35LayerType::FullAttention);
    }

    #[test]
    fn parses_official_qwen35_4b_config_shape() {
        let bytes = br#"{
          "model_type": "qwen3_5",
          "text_config": {
            "model_type": "qwen3_5_text",
            "vocab_size": 248320,
            "hidden_size": 2560,
            "intermediate_size": 9216,
            "num_hidden_layers": 32,
            "num_attention_heads": 16,
            "num_key_value_heads": 4,
            "head_dim": 256,
            "linear_conv_kernel_dim": 4,
            "linear_key_head_dim": 128,
            "linear_value_head_dim": 128,
            "linear_num_key_heads": 16,
            "linear_num_value_heads": 32,
            "layer_types": ["linear_attention", "linear_attention", "linear_attention", "full_attention",
              "linear_attention", "linear_attention", "linear_attention", "full_attention",
              "linear_attention", "linear_attention", "linear_attention", "full_attention",
              "linear_attention", "linear_attention", "linear_attention", "full_attention",
              "linear_attention", "linear_attention", "linear_attention", "full_attention",
              "linear_attention", "linear_attention", "linear_attention", "full_attention",
              "linear_attention", "linear_attention", "linear_attention", "full_attention",
              "linear_attention", "linear_attention", "linear_attention", "full_attention"],
            "rms_norm_eps": 1e-6,
            "max_position_embeddings": 262144,
            "hidden_act": "silu",
            "eos_token_id": 248044,
            "rope_parameters": {
              "rope_type": "default",
              "rope_theta": 10000000,
              "partial_rotary_factor": 0.25
            }
          }
        }"#;
        let cfg = parse_config_bytes(bytes).unwrap();
        assert_eq!(cfg.hidden_size, 2560);
        assert_eq!(cfg.num_hidden_layers, 32);
        assert_eq!(cfg.linear_conv_dim(), 8192);
        assert_eq!(cfg.rotary_dim(), 64);
        assert_eq!(
            cfg.layer_types
                .iter()
                .filter(|layer| **layer == Qwen35LayerType::FullAttention)
                .count(),
            8
        );
    }

    #[test]
    fn tiny_qwen35_fixture_loads_and_generates() -> Result<()> {
        let temp = tempfile::tempdir()?;
        write_tiny_qwen35_fixture(temp.path())?;
        let model = Qwen35TextModel::from_hf_dir(temp.path())?;
        assert_eq!(model.cfg.num_hidden_layers, 2);
        let mut cache = model.new_cache_with_capacity(4);
        let logits = model.forward_token(2, 0, &mut cache);
        assert_eq!(logits.len(), 16);
        let output = model.generate_greedy(&[2], 2);
        assert_eq!(output.len(), 3);
        Ok(())
    }

    #[test]
    fn tiny_qwen35_fixture_loads_q4_cache_and_generates() -> Result<()> {
        let temp = tempfile::tempdir()?;
        let cache = tempfile::tempdir()?;
        write_tiny_qwen35_fixture(temp.path())?;
        let first = Qwen35TextModel::from_hf_dir_with_cache_and_mode(
            temp.path(),
            cache.path(),
            QuantMode::Q4,
        )?;
        assert!(matches!(first.token_embedding, LinearMatrix::Quant(_)));
        let cache_files = fs::read_dir(cache.path())?
            .filter_map(Result::ok)
            .filter(|entry| entry.path().extension().and_then(|ext| ext.to_str()) == Some("zq4"))
            .count();
        assert!(cache_files > 0);

        let second = Qwen35TextModel::from_hf_dir_with_cache_and_mode(
            temp.path(),
            cache.path(),
            QuantMode::Q4,
        )?;
        let mut cache_state = second.new_cache_with_capacity(4);
        let logits = second.forward_token(2, 0, &mut cache_state);
        assert_eq!(logits.len(), 16);
        let output = second.generate_greedy(&[2], 2);
        assert_eq!(output.len(), 3);
        Ok(())
    }

    fn write_tiny_qwen35_fixture(dir: &Path) -> Result<()> {
        fs::write(
            dir.join("config.json"),
            serde_json::to_vec_pretty(&serde_json::json!({
                "model_type": "qwen3_5",
                "text_config": {
                    "model_type": "qwen3_5_text",
                    "vocab_size": 16,
                    "hidden_size": 8,
                    "intermediate_size": 16,
                    "num_hidden_layers": 2,
                    "num_attention_heads": 2,
                    "num_key_value_heads": 1,
                    "head_dim": 4,
                    "linear_conv_kernel_dim": 4,
                    "linear_key_head_dim": 2,
                    "linear_value_head_dim": 2,
                    "linear_num_key_heads": 1,
                    "linear_num_value_heads": 1,
                    "layer_types": ["linear_attention", "full_attention"],
                    "rms_norm_eps": 1e-6,
                    "max_position_embeddings": 16,
                    "hidden_act": "silu",
                    "eos_token_id": 15,
                    "rope_parameters": {
                        "rope_type": "default",
                        "rope_theta": 10000.0,
                        "partial_rotary_factor": 1.0
                    }
                }
            }))?,
        )?;

        let mut tensors = Vec::<(String, TestTensor)>::new();
        push_matrix(
            &mut tensors,
            "model.language_model.embed_tokens.weight",
            16,
            8,
            patterned(16 * 8, 0.02),
        );
        push_vec(
            &mut tensors,
            "model.language_model.norm.weight",
            vec![0.0; 8],
        );

        for layer in 0..2 {
            let prefix = format!("model.language_model.layers.{layer}");
            push_vec(
                &mut tensors,
                &format!("{prefix}.input_layernorm.weight"),
                vec![0.0; 8],
            );
            push_vec(
                &mut tensors,
                &format!("{prefix}.post_attention_layernorm.weight"),
                vec![0.0; 8],
            );
            push_matrix(
                &mut tensors,
                &format!("{prefix}.mlp.gate_proj.weight"),
                16,
                8,
                patterned(16 * 8, 0.01 + layer as f32 * 0.001),
            );
            push_matrix(
                &mut tensors,
                &format!("{prefix}.mlp.up_proj.weight"),
                16,
                8,
                patterned(16 * 8, 0.012 + layer as f32 * 0.001),
            );
            push_matrix(
                &mut tensors,
                &format!("{prefix}.mlp.down_proj.weight"),
                8,
                16,
                patterned(8 * 16, 0.009 + layer as f32 * 0.001),
            );
        }

        let prefix = "model.language_model.layers.0";
        push_matrix(
            &mut tensors,
            &format!("{prefix}.linear_attn.in_proj_qkv.weight"),
            6,
            8,
            patterned(6 * 8, 0.015),
        );
        push_matrix(
            &mut tensors,
            &format!("{prefix}.linear_attn.in_proj_z.weight"),
            2,
            8,
            patterned(2 * 8, 0.013),
        );
        push_matrix(
            &mut tensors,
            &format!("{prefix}.linear_attn.in_proj_b.weight"),
            1,
            8,
            patterned(8, 0.011),
        );
        push_matrix(
            &mut tensors,
            &format!("{prefix}.linear_attn.in_proj_a.weight"),
            1,
            8,
            patterned(8, 0.007),
        );
        push_tensor(
            &mut tensors,
            &format!("{prefix}.linear_attn.conv1d.weight"),
            vec![6, 1, 4],
            patterned(6 * 4, 0.02),
        );
        push_vec(
            &mut tensors,
            &format!("{prefix}.linear_attn.dt_bias"),
            vec![0.1],
        );
        push_vec(
            &mut tensors,
            &format!("{prefix}.linear_attn.A_log"),
            vec![0.0],
        );
        push_vec(
            &mut tensors,
            &format!("{prefix}.linear_attn.norm.weight"),
            vec![1.0; 2],
        );
        push_matrix(
            &mut tensors,
            &format!("{prefix}.linear_attn.out_proj.weight"),
            8,
            2,
            patterned(16, 0.014),
        );

        let prefix = "model.language_model.layers.1";
        push_matrix(
            &mut tensors,
            &format!("{prefix}.self_attn.q_proj.weight"),
            16,
            8,
            patterned(16 * 8, 0.016),
        );
        push_matrix(
            &mut tensors,
            &format!("{prefix}.self_attn.k_proj.weight"),
            4,
            8,
            patterned(4 * 8, 0.017),
        );
        push_matrix(
            &mut tensors,
            &format!("{prefix}.self_attn.v_proj.weight"),
            4,
            8,
            patterned(4 * 8, 0.018),
        );
        push_matrix(
            &mut tensors,
            &format!("{prefix}.self_attn.o_proj.weight"),
            8,
            8,
            patterned(64, 0.019),
        );
        push_vec(
            &mut tensors,
            &format!("{prefix}.self_attn.q_norm.weight"),
            vec![0.0; 4],
        );
        push_vec(
            &mut tensors,
            &format!("{prefix}.self_attn.k_norm.weight"),
            vec![0.0; 4],
        );

        serialize_to_file(tensors, &None, &dir.join("model.safetensors"))?;
        Ok(())
    }

    fn push_matrix(
        tensors: &mut Vec<(String, TestTensor)>,
        name: &str,
        rows: usize,
        cols: usize,
        data: Vec<f32>,
    ) {
        push_tensor(tensors, name, vec![rows, cols], data);
    }

    fn push_vec(tensors: &mut Vec<(String, TestTensor)>, name: &str, data: Vec<f32>) {
        push_tensor(tensors, name, vec![data.len()], data);
    }

    fn push_tensor(
        tensors: &mut Vec<(String, TestTensor)>,
        name: &str,
        shape: Vec<usize>,
        data: Vec<f32>,
    ) {
        tensors.push((
            name.to_string(),
            TestTensor {
                shape,
                bytes: f32_bytes(&data),
            },
        ));
    }

    fn patterned(len: usize, scale: f32) -> Vec<f32> {
        (0..len)
            .map(|idx| ((idx % 17) as f32 - 8.0) * scale)
            .collect()
    }

    fn f32_bytes(values: &[f32]) -> Vec<u8> {
        values
            .iter()
            .flat_map(|value| value.to_le_bytes())
            .collect()
    }
}
