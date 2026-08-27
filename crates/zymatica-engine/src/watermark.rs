//! Cryptographic token watermarking for tamper-evident agent output logs.

use anyhow::{Context, Result, bail};
use ed25519_dalek::{Signature, Signer, SigningKey, Verifier, VerifyingKey};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct WatermarkConfig {
    pub top_k: usize,
    pub equivalence_delta: f32,
    pub strength: f32,
}

impl Default for WatermarkConfig {
    fn default() -> Self {
        Self {
            top_k: 8,
            equivalence_delta: 0.05,
            strength: 0.01,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct WatermarkStep {
    pub position: usize,
    pub selected_token: usize,
    pub candidate_count: usize,
    pub greenlisted: bool,
    pub watermark_score: u64,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TokenWatermarkLog {
    pub public_key_hex: String,
    pub context_hash: String,
    pub signature_hex: String,
    pub config: WatermarkConfig,
    pub steps: Vec<WatermarkStep>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct WatermarkVerification {
    pub checked_steps: usize,
    pub watermark_hits: usize,
    pub hit_rate: f32,
}

pub struct WatermarkSigner {
    signing_key: SigningKey,
    public_key_hex: String,
}

impl WatermarkSigner {
    pub fn from_seed(seed: [u8; 32]) -> Self {
        let signing_key = SigningKey::from_bytes(&seed);
        let public_key_hex = hex_encode(&signing_key.verifying_key().to_bytes());
        Self {
            signing_key,
            public_key_hex,
        }
    }

    pub fn public_key_hex(&self) -> &str {
        &self.public_key_hex
    }

    pub fn start_log(&self, context: &[u8], config: WatermarkConfig) -> TokenWatermarkLog {
        let context_hash_bytes = Sha256::digest(context);
        let signature = self.signing_key.sign(context_hash_bytes.as_slice());
        TokenWatermarkLog {
            public_key_hex: self.public_key_hex.clone(),
            context_hash: hex_encode(context_hash_bytes.as_slice()),
            signature_hex: hex_encode(&signature.to_bytes()),
            config,
            steps: Vec::new(),
        }
    }

    pub fn select_step(
        &self,
        logits: &[f32],
        context: &[u8],
        position: usize,
        config: WatermarkConfig,
    ) -> Result<WatermarkStep> {
        let signature = self.signing_key.sign(Sha256::digest(context).as_slice());
        select_from_signature(logits, position, config, &signature.to_bytes())
    }

    pub fn append_step(
        &self,
        log: &mut TokenWatermarkLog,
        logits: &[f32],
        position: usize,
    ) -> Result<usize> {
        let signature = hex_decode_fixed::<64>(&log.signature_hex)?;
        let step = select_from_signature(logits, position, log.config, &signature)?;
        let selected_token = step.selected_token;
        log.steps.push(step);
        Ok(selected_token)
    }
}

pub fn verify_watermark_log(
    context: &[u8],
    logits_by_step: &[Vec<f32>],
    log: &TokenWatermarkLog,
) -> Result<WatermarkVerification> {
    validate_config(log.config)?;
    if logits_by_step.len() != log.steps.len() {
        bail!(
            "watermark step count mismatch: logits={} log={}",
            logits_by_step.len(),
            log.steps.len()
        );
    }

    let context_hash = Sha256::digest(context);
    let context_hash_hex = hex_encode(context_hash.as_slice());
    if context_hash_hex != log.context_hash {
        bail!("watermark context hash mismatch");
    }

    let public_key = VerifyingKey::from_bytes(&hex_decode_fixed::<32>(&log.public_key_hex)?)?;
    let signature_bytes = hex_decode_fixed::<64>(&log.signature_hex)?;
    let signature = Signature::from_bytes(&signature_bytes);
    public_key
        .verify(context_hash.as_slice(), &signature)
        .context("verifying token watermark signature")?;

    let mut hits = 0;
    for (idx, (logits, observed)) in logits_by_step.iter().zip(&log.steps).enumerate() {
        let expected =
            select_from_signature(logits, observed.position, log.config, &signature_bytes)
                .with_context(|| format!("recomputing watermark step {idx}"))?;
        if expected != *observed {
            bail!(
                "watermark step {idx} mismatch: expected {:?} observed {:?}",
                expected,
                observed
            );
        }
        if observed.greenlisted {
            hits += 1;
        }
    }

    let checked_steps = log.steps.len();
    Ok(WatermarkVerification {
        checked_steps,
        watermark_hits: hits,
        hit_rate: if checked_steps == 0 {
            0.0
        } else {
            hits as f32 / checked_steps as f32
        },
    })
}

fn select_from_signature(
    logits: &[f32],
    position: usize,
    config: WatermarkConfig,
    signature: &[u8; 64],
) -> Result<WatermarkStep> {
    validate_config(config)?;
    let candidates = equivalent_candidates(logits, config)?;
    let mut best = None;
    for (token_id, logit) in &candidates {
        let watermark_score = watermark_score(signature, position, *token_id);
        let normalized = watermark_score as f32 / u64::MAX as f32;
        let combined = *logit + config.strength * normalized;
        match best {
            Some((_, best_combined, best_score)) => {
                if combined > best_combined
                    || (combined == best_combined && watermark_score > best_score)
                {
                    best = Some((*token_id, combined, watermark_score));
                }
            }
            None => best = Some((*token_id, combined, watermark_score)),
        }
    }
    let (selected_token, _, watermark_score) =
        best.context("watermark candidate selection produced no token")?;
    Ok(WatermarkStep {
        position,
        selected_token,
        candidate_count: candidates.len(),
        greenlisted: watermark_score >= (u64::MAX / 2),
        watermark_score,
    })
}

fn equivalent_candidates(logits: &[f32], config: WatermarkConfig) -> Result<Vec<(usize, f32)>> {
    if logits.is_empty() {
        bail!("watermark logits must not be empty");
    }
    let mut ranked: Vec<_> = logits
        .iter()
        .copied()
        .enumerate()
        .filter(|(_, logit)| logit.is_finite())
        .collect();
    if ranked.is_empty() {
        bail!("watermark logits contain no finite candidates");
    }
    ranked.sort_by(|a, b| b.1.total_cmp(&a.1).then_with(|| a.0.cmp(&b.0)));
    ranked.truncate(config.top_k.min(ranked.len()));
    let max = ranked[0].1;
    ranked.retain(|(_, logit)| *logit >= max - config.equivalence_delta);
    Ok(ranked)
}

fn validate_config(config: WatermarkConfig) -> Result<()> {
    if config.top_k == 0 {
        bail!("watermark top_k must be greater than zero");
    }
    if config.equivalence_delta < 0.0 || !config.equivalence_delta.is_finite() {
        bail!("watermark equivalence_delta must be finite and non-negative");
    }
    if config.strength < 0.0 || !config.strength.is_finite() {
        bail!("watermark strength must be finite and non-negative");
    }
    Ok(())
}

fn watermark_score(signature: &[u8; 64], position: usize, token_id: usize) -> u64 {
    let mut hasher = Sha256::new();
    hasher.update(signature);
    hasher.update((position as u64).to_le_bytes());
    hasher.update((token_id as u64).to_le_bytes());
    let digest = hasher.finalize();
    u64::from_le_bytes(digest[..8].try_into().expect("sha256 digest has 32 bytes"))
}

fn hex_encode(bytes: &[u8]) -> String {
    const LUT: &[u8; 16] = b"0123456789abcdef";
    let mut out = String::with_capacity(bytes.len() * 2);
    for byte in bytes {
        out.push(LUT[(byte >> 4) as usize] as char);
        out.push(LUT[(byte & 0x0f) as usize] as char);
    }
    out
}

fn hex_decode_fixed<const N: usize>(hex: &str) -> Result<[u8; N]> {
    if hex.len() != N * 2 {
        bail!("hex length {} does not match {} bytes", hex.len(), N);
    }
    let mut out = [0_u8; N];
    let bytes = hex.as_bytes();
    for idx in 0..N {
        out[idx] = (hex_nibble(bytes[idx * 2])? << 4) | hex_nibble(bytes[idx * 2 + 1])?;
    }
    Ok(out)
}

fn hex_nibble(byte: u8) -> Result<u8> {
    match byte {
        b'0'..=b'9' => Ok(byte - b'0'),
        b'a'..=b'f' => Ok(byte - b'a' + 10),
        b'A'..=b'F' => Ok(byte - b'A' + 10),
        _ => bail!("invalid hex byte {}", byte),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn watermark_selection_is_publicly_verifiable() {
        let signer = WatermarkSigner::from_seed([11_u8; 32]);
        let config = WatermarkConfig {
            top_k: 4,
            equivalence_delta: 0.1,
            strength: 0.02,
        };
        let context = b"agent command chain #17";
        let mut log = signer.start_log(context, config);
        let logits_by_step = vec![
            vec![1.0, 1.0, 1.0, 0.7, -4.0],
            vec![0.2, 0.8, 0.8, 0.8, -1.0],
        ];
        for (position, logits) in logits_by_step.iter().enumerate() {
            signer.append_step(&mut log, logits, position).unwrap();
        }

        let verified = verify_watermark_log(context, &logits_by_step, &log).unwrap();
        assert_eq!(verified.checked_steps, 2);
        assert_eq!(log.public_key_hex, signer.public_key_hex());
    }

    #[test]
    fn watermark_verification_rejects_tampered_tokens() {
        let signer = WatermarkSigner::from_seed([12_u8; 32]);
        let context = b"agent audit transcript";
        let logits_by_step = vec![vec![3.0, 3.0, 3.0, 0.0]];
        let mut log = signer.start_log(context, WatermarkConfig::default());
        signer.append_step(&mut log, &logits_by_step[0], 0).unwrap();
        log.steps[0].selected_token = (log.steps[0].selected_token + 1) % 3;
        assert!(verify_watermark_log(context, &logits_by_step, &log).is_err());
    }
}
