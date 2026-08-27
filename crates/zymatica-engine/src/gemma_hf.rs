use crate::model::GemmaConfig;
use crate::weights::TensorIndex;
use anyhow::{Context, Result, bail};
use serde::Deserialize;
use serde_json::Value;
use std::fs;
use std::path::Path;

#[derive(Debug, Clone, Deserialize)]
struct RawGemmaConfig {
    model_type: Option<String>,
    vocab_size: Option<usize>,
    hidden_size: Option<usize>,
    intermediate_size: Option<usize>,
    num_hidden_layers: Option<usize>,
    num_attention_heads: Option<usize>,
    num_key_value_heads: Option<usize>,
    num_kv_shared_layers: Option<usize>,
    head_dim: Option<usize>,
    global_head_dim: Option<usize>,
    rms_norm_eps: Option<f32>,
    rope_theta: Option<f32>,
    rope_parameters: Option<Value>,
    max_position_embeddings: Option<usize>,
    sliding_window: Option<usize>,
    layer_types: Option<Vec<String>>,
    hidden_size_per_layer_input: Option<usize>,
    vocab_size_per_layer_input: Option<usize>,
    hidden_activation: Option<String>,
    final_logit_softcapping: Option<f32>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ResolvedTensor {
    pub role: String,
    pub name: Option<String>,
}

#[derive(Debug, Clone)]
pub struct GemmaResolution {
    pub config: GemmaConfig,
    pub tensor_count: usize,
    pub tensors: Vec<ResolvedTensor>,
}

impl GemmaResolution {
    pub fn missing(&self) -> Vec<&ResolvedTensor> {
        self.tensors
            .iter()
            .filter(|tensor| tensor.name.is_none())
            .collect()
    }
}

pub fn parse_config_file(path: impl AsRef<Path>) -> Result<GemmaConfig> {
    let path = path.as_ref();
    let value: Value = serde_json::from_slice(
        &fs::read(path).with_context(|| format!("reading {}", path.display()))?,
    )
    .with_context(|| format!("parsing {}", path.display()))?;

    let config_value = value.get("text_config").cloned().unwrap_or(value);
    let raw: RawGemmaConfig = serde_json::from_value(config_value)?;
    raw.into_config()
}

pub fn parse_config_bytes(bytes: &[u8]) -> Result<GemmaConfig> {
    let value: Value = serde_json::from_slice(bytes).context("parsing config bytes")?;

    let config_value = value.get("text_config").cloned().unwrap_or(value);
    let raw: RawGemmaConfig = serde_json::from_value(config_value)?;
    raw.into_config()
}

pub fn resolve_gemma_dir(model_dir: impl AsRef<Path>) -> Result<GemmaResolution> {
    let model_dir = model_dir.as_ref();
    let config = parse_config_file(model_dir.join("config.json"))?;
    let index = TensorIndex::from_dir(model_dir)?;
    let tensors = resolve_tensor_names(&config, &index);
    Ok(GemmaResolution {
        config,
        tensor_count: index.len(),
        tensors,
    })
}

pub fn resolve_gemma_in_memory(
    config_bytes: &[u8],
    index: &TensorIndex,
) -> Result<GemmaResolution> {
    let config = parse_config_bytes(config_bytes)?;
    let tensors = resolve_tensor_names(&config, index);
    Ok(GemmaResolution {
        config,
        tensor_count: index.len(),
        tensors,
    })
}

pub fn resolve_tensor_names(config: &GemmaConfig, index: &TensorIndex) -> Vec<ResolvedTensor> {
    let mut out = Vec::new();
    push_resolved(
        &mut out,
        index,
        "token_embedding",
        &[
            "model.embed_tokens.weight",
            "model.language_model.embed_tokens.weight",
            "language_model.embed_tokens.weight",
        ],
    );
    push_resolved(
        &mut out,
        index,
        "token_embedding_per_layer",
        &[
            "model.embed_tokens_per_layer.weight",
            "model.language_model.embed_tokens_per_layer.weight",
            "language_model.embed_tokens_per_layer.weight",
        ],
    );
    push_resolved(
        &mut out,
        index,
        "per_layer_model_projection",
        &[
            "model.per_layer_model_projection.weight",
            "model.language_model.per_layer_model_projection.weight",
            "language_model.per_layer_model_projection.weight",
        ],
    );
    push_resolved(
        &mut out,
        index,
        "per_layer_projection_norm",
        &[
            "model.per_layer_projection_norm.weight",
            "model.language_model.per_layer_projection_norm.weight",
            "language_model.per_layer_projection_norm.weight",
        ],
    );
    push_resolved(
        &mut out,
        index,
        "final_norm",
        &[
            "model.norm.weight",
            "model.language_model.norm.weight",
            "language_model.norm.weight",
        ],
    );
    push_resolved(
        &mut out,
        index,
        "lm_head",
        &[
            "lm_head.weight",
            "model.lm_head.weight",
            "model.language_model.lm_head.weight",
            "language_model.lm_head.weight",
            "model.embed_tokens.weight",
            "model.language_model.embed_tokens.weight",
        ],
    );

    for layer in 0..config.num_hidden_layers {
        for (role, suffixes) in [
            (
                "input_norm",
                &["input_layernorm.weight", "pre_attention_layernorm.weight"][..],
            ),
            (
                "post_attention_norm",
                &["post_attention_layernorm.weight"][..],
            ),
            (
                "pre_feedforward_norm",
                &["pre_feedforward_layernorm.weight"][..],
            ),
            (
                "post_feedforward_norm",
                &["post_feedforward_layernorm.weight"][..],
            ),
            ("q_norm", &["self_attn.q_norm.weight"][..]),
            ("k_norm", &["self_attn.k_norm.weight"][..]),
            ("q_proj", &["self_attn.q_proj.weight"][..]),
            ("k_proj", &["self_attn.k_proj.weight"][..]),
            ("v_proj", &["self_attn.v_proj.weight"][..]),
            ("o_proj", &["self_attn.o_proj.weight"][..]),
            (
                "gate_proj",
                &["mlp.gate_proj.weight", "mlp.gate.weight"][..],
            ),
            ("up_proj", &["mlp.up_proj.weight", "mlp.up.weight"][..]),
            (
                "down_proj",
                &["mlp.down_proj.weight", "mlp.down.weight"][..],
            ),
            ("layer_scalar", &["layer_scalar"][..]),
            ("per_layer_input_gate", &["per_layer_input_gate.weight"][..]),
            ("per_layer_projection", &["per_layer_projection.weight"][..]),
            (
                "post_per_layer_input_norm",
                &["post_per_layer_input_norm.weight"][..],
            ),
        ] {
            let candidates = layer_candidates(layer, suffixes);
            let candidate_refs: Vec<_> = candidates.iter().map(String::as_str).collect();
            push_resolved(
                &mut out,
                index,
                &format!("layers.{layer}.{role}"),
                &candidate_refs,
            );
        }
    }

    out
}

fn push_resolved(
    out: &mut Vec<ResolvedTensor>,
    index: &TensorIndex,
    role: &str,
    candidates: &[&str],
) {
    out.push(ResolvedTensor {
        role: role.to_owned(),
        name: index.find_first(candidates.iter().copied()),
    });
}

fn layer_candidates(layer: usize, suffixes: &[&str]) -> Vec<String> {
    let bases = [
        format!("model.layers.{layer}"),
        format!("model.language_model.layers.{layer}"),
        format!("language_model.layers.{layer}"),
    ];
    bases
        .iter()
        .flat_map(|base| {
            suffixes
                .iter()
                .map(move |suffix| format!("{base}.{suffix}"))
        })
        .collect()
}

impl RawGemmaConfig {
    fn into_config(self) -> Result<GemmaConfig> {
        let vocab_size = required(self.vocab_size, "vocab_size")?;
        let hidden_size = required(self.hidden_size, "hidden_size")?;
        let intermediate_size = required(self.intermediate_size, "intermediate_size")?;
        let num_hidden_layers = required(self.num_hidden_layers, "num_hidden_layers")?;
        let num_attention_heads = required(self.num_attention_heads, "num_attention_heads")?;
        let num_key_value_heads = self.num_key_value_heads.unwrap_or(num_attention_heads);
        let head_dim = self
            .head_dim
            .unwrap_or_else(|| hidden_size / num_attention_heads);
        if num_attention_heads % num_key_value_heads != 0 {
            bail!(
                "invalid Gemma config: attention heads {} are not divisible by kv heads {}",
                num_attention_heads,
                num_key_value_heads
            );
        }

        let rope_theta = self
            .rope_theta
            .or_else(|| rope_value(&self.rope_parameters, "sliding_attention", "rope_theta"))
            .unwrap_or(10_000.0);
        let full_attention_rope_theta =
            rope_value(&self.rope_parameters, "full_attention", "rope_theta");
        let full_attention_rotary_fraction = rope_value(
            &self.rope_parameters,
            "full_attention",
            "partial_rotary_factor",
        )
        .unwrap_or(1.0);

        let layer_types = match self.layer_types {
            Some(layer_types) if layer_types.len() == num_hidden_layers => layer_types,
            Some(layer_types) => {
                bail!(
                    "invalid Gemma config: layer_types len {} != num_hidden_layers {}",
                    layer_types.len(),
                    num_hidden_layers
                );
            }
            None => vec!["sliding_attention".to_string(); num_hidden_layers],
        };

        let is_gemma2_or_4 = self
            .model_type
            .as_deref()
            .map(|model_type| {
                model_type.starts_with("gemma4")
                    || model_type.starts_with("gemma2")
                    || model_type.contains("gemma-2")
            })
            .unwrap_or(false)
            || self.hidden_size_per_layer_input.is_some();
        let hidden_size_per_layer_input = self.hidden_size_per_layer_input;
        let embedding_scale = if is_gemma2_or_4 {
            (hidden_size as f32).sqrt()
        } else {
            1.0
        };
        let per_layer_embedding_scale = hidden_size_per_layer_input
            .map(|dim| (dim as f32).sqrt())
            .unwrap_or(1.0);

        if hidden_size != num_attention_heads * head_dim && !is_gemma2_or_4 {
            bail!(
                "invalid legacy Gemma config: hidden_size={} num_attention_heads={} head_dim={}",
                hidden_size,
                num_attention_heads,
                head_dim
            );
        }
        Ok(GemmaConfig {
            vocab_size,
            hidden_size,
            intermediate_size,
            num_hidden_layers,
            num_attention_heads,
            num_key_value_heads,
            num_kv_shared_layers: self.num_kv_shared_layers.unwrap_or(0),
            head_dim,
            global_head_dim: self.global_head_dim,
            rms_norm_eps: self.rms_norm_eps.unwrap_or(1e-6),
            rope_theta,
            full_attention_rope_theta,
            full_attention_rotary_fraction,
            max_position_embeddings: self.max_position_embeddings.unwrap_or(8192),
            sliding_window: self.sliding_window,
            layer_types,
            hidden_size_per_layer_input,
            vocab_size_per_layer_input: self.vocab_size_per_layer_input,
            hidden_activation: self.hidden_activation.unwrap_or_else(|| "silu".to_string()),
            embedding_scale,
            per_layer_embedding_scale,
            final_logit_softcapping: self.final_logit_softcapping,
            fold_rms_norm: false,
        })
    }
}

fn required<T>(value: Option<T>, name: &str) -> Result<T> {
    value.with_context(|| format!("missing required Gemma config field: {name}"))
}

fn rope_value(rope_parameters: &Option<Value>, layer_type: &str, key: &str) -> Option<f32> {
    rope_parameters
        .as_ref()
        .and_then(|value| value.get(layer_type).or(Some(value)))
        .and_then(|value| value.get(key))
        .and_then(Value::as_f64)
        .map(|value| value as f32)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_top_level_config() {
        let raw = RawGemmaConfig {
            model_type: Some("gemma".to_string()),
            vocab_size: Some(256000),
            hidden_size: Some(2048),
            intermediate_size: Some(8192),
            num_hidden_layers: Some(26),
            num_attention_heads: Some(8),
            num_key_value_heads: Some(4),
            num_kv_shared_layers: None,
            head_dim: Some(256),
            global_head_dim: None,
            rms_norm_eps: Some(1e-6),
            rope_theta: Some(1_000_000.0),
            rope_parameters: None,
            max_position_embeddings: Some(131072),
            sliding_window: None,
            layer_types: None,
            hidden_size_per_layer_input: None,
            vocab_size_per_layer_input: None,
            hidden_activation: None,
            final_logit_softcapping: None,
        };
        let cfg = raw.into_config().unwrap();
        assert_eq!(cfg.hidden_size, 2048);
        assert_eq!(cfg.num_key_value_heads, 4);
        assert_eq!(cfg.head_dim, 256);
    }

    #[test]
    fn rejects_inconsistent_head_dim() {
        let raw = RawGemmaConfig {
            model_type: Some("gemma".to_string()),
            vocab_size: Some(10),
            hidden_size: Some(16),
            intermediate_size: Some(32),
            num_hidden_layers: Some(2),
            num_attention_heads: Some(3),
            num_key_value_heads: None,
            num_kv_shared_layers: None,
            head_dim: Some(4),
            global_head_dim: None,
            rms_norm_eps: None,
            rope_theta: None,
            rope_parameters: None,
            max_position_embeddings: None,
            sliding_window: None,
            layer_types: None,
            hidden_size_per_layer_input: None,
            vocab_size_per_layer_input: None,
            hidden_activation: None,
            final_logit_softcapping: None,
        };
        assert!(raw.into_config().is_err());
    }

    #[test]
    fn accepts_gemma4_mixed_attention_config() {
        let raw = RawGemmaConfig {
            model_type: Some("gemma4_text".to_string()),
            vocab_size: Some(262144),
            hidden_size: Some(1536),
            intermediate_size: Some(6144),
            num_hidden_layers: Some(2),
            num_attention_heads: Some(8),
            num_key_value_heads: Some(1),
            num_kv_shared_layers: Some(1),
            head_dim: Some(256),
            global_head_dim: Some(512),
            rms_norm_eps: Some(1e-6),
            rope_theta: None,
            rope_parameters: Some(serde_json::json!({
                "sliding_attention": {"rope_theta": 10000.0, "rope_type": "default"},
                "full_attention": {
                    "rope_theta": 1000000.0,
                    "rope_type": "proportional",
                    "partial_rotary_factor": 0.25
                }
            })),
            max_position_embeddings: Some(131072),
            sliding_window: Some(512),
            layer_types: Some(vec![
                "sliding_attention".to_string(),
                "full_attention".to_string(),
            ]),
            hidden_size_per_layer_input: Some(256),
            vocab_size_per_layer_input: Some(262144),
            hidden_activation: Some("gelu_pytorch_tanh".to_string()),
            final_logit_softcapping: Some(30.0),
        };
        let cfg = raw.into_config().unwrap();
        assert_eq!(cfg.hidden_size, 1536);
        assert_eq!(cfg.head_dim, 256);
        assert_eq!(cfg.full_attention_rope_theta, Some(1_000_000.0));
        assert_eq!(cfg.full_attention_rotary_fraction, 0.25);
        assert_eq!(cfg.embedding_scale, (1536.0_f32).sqrt());
        assert_eq!(cfg.num_kv_shared_layers, 1);
        assert_eq!(cfg.hidden_activation, "gelu_pytorch_tanh");
    }
}
