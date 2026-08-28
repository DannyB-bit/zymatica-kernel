//! ==============================================================================
//! ZYMATICA CLASS 33: Z-SPAR (Zymatica Semantic Parity and Repair Protocol)
//! Author: Danny Bouldiez | Codebase by Devs One
//!
//! Model-Independent Semantic Error-Detection, Error-Correction, and Incremental Repair.
//! Computes finite-field parity over GF(16) discrete semantic manifold coordinates rather
//! than raw character tokens, allowing heterogeneous LLMs (Qwen, Gemma, SmolLM) to detect
//! and automatically correct semantic drift without retransmitting natural language prompts.
//! ==============================================================================

use std::collections::HashMap;

/// Galois Field GF(16) Arithmetic using primitive polynomial p(x) = x^4 + x + 1 (0x13 / 19 in decimal)
pub struct GF16;

impl GF16 {
    // Exp and Log tables for GF(16) generated with generator alpha = 2 (x)
    pub const EXP: [u8; 32] = [
        1, 2, 4, 8, 3, 6, 12, 11, 5, 10, 7, 14, 15, 13, 9,
        1, // Repeat for fast modular indexing
        2, 4, 8, 3, 6, 12, 11, 5, 10, 7, 14, 15, 13, 9, 1, 2,
    ];

    pub const LOG: [u8; 16] = [0, 0, 1, 4, 2, 8, 5, 10, 3, 14, 9, 7, 6, 13, 11, 12];

    #[inline]
    pub fn add(a: u8, b: u8) -> u8 {
        (a ^ b) & 0x0F
    }

    #[inline]
    pub fn mul(a: u8, b: u8) -> u8 {
        let a = a & 0x0F;
        let b = b & 0x0F;
        if a == 0 || b == 0 {
            0
        } else {
            let log_sum = (Self::LOG[a as usize] as usize) + (Self::LOG[b as usize] as usize);
            Self::EXP[log_sum % 15]
        }
    }

    #[inline]
    pub fn div(a: u8, b: u8) -> Result<u8, &'static str> {
        let a = a & 0x0F;
        let b = b & 0x0F;
        if b == 0 {
            return Err("Division by zero in GF(16)");
        }
        if a == 0 {
            return Ok(0);
        }
        let log_diff = (Self::LOG[a as usize] as i32) - (Self::LOG[b as usize] as i32) + 15;
        Ok(Self::EXP[(log_diff as usize) % 15])
    }

    #[inline]
    pub fn power(a: u8, exp: usize) -> u8 {
        let a = a & 0x0F;
        if a == 0 {
            return 0;
        }
        let log_a = Self::LOG[a as usize] as usize;
        Self::EXP[(log_a * exp) % 15]
    }
}

/// 8D Language-U Semantic State Coordinate Vector
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Concept8DState {
    pub domain: u8,              // D  (0..15)
    pub subdomain: u8,           // SD (0..15)
    pub operation: u8,           // OP (0..15)
    pub modality: u8,            // M  (0..15)
    pub strength: u8,            // S  (0..15)
    pub polarity: u8,            // P  (0..15)
    pub temporal_horizon: u8,    // T  (0..15)
    pub epistemic_certainty: u8, // E  (0..15)
}

impl Concept8DState {
    pub fn new(d: u8, sd: u8, op: u8, m: u8, s: u8, p: u8, t: u8, e: u8) -> Self {
        Self {
            domain: d & 0x0F,
            subdomain: sd & 0x0F,
            operation: op & 0x0F,
            modality: m & 0x0F,
            strength: s & 0x0F,
            polarity: p & 0x0F,
            temporal_horizon: t & 0x0F,
            epistemic_certainty: e & 0x0F,
        }
    }

    pub fn to_symbols(&self) -> [u8; 8] {
        [
            self.domain,
            self.subdomain,
            self.operation,
            self.modality,
            self.strength,
            self.polarity,
            self.temporal_horizon,
            self.epistemic_certainty,
        ]
    }

    pub fn from_symbols(syms: &[u8; 8]) -> Self {
        Self::new(
            syms[0], syms[1], syms[2], syms[3], syms[4], syms[5], syms[6], syms[7],
        )
    }

    pub fn to_zspar(&self) -> zymatica_zspar::Concept8D {
        zymatica_zspar::Concept8D::new(
            self.domain,
            self.subdomain,
            self.operation,
            self.modality,
            self.strength,
            self.polarity,
            self.temporal_horizon,
            self.epistemic_certainty,
        )
    }

    pub fn from_zspar(c: zymatica_zspar::Concept8D) -> Self {
        Self::new(
            c.domain,
            c.subdomain,
            c.operation,
            c.modality,
            c.strength,
            c.polarity,
            c.temporal_horizon,
            c.epistemic_certainty,
        )
    }

    pub fn to_dword(&self) -> u32 {
        ((self.domain as u32) << 28)
            | ((self.subdomain as u32) << 24)
            | ((self.operation as u32) << 20)
            | ((self.modality as u32) << 16)
            | ((self.strength as u32) << 12)
            | ((self.polarity as u32) << 8)
            | ((self.temporal_horizon as u32) << 4)
            | (self.epistemic_certainty as u32 & 0x0F)
    }

    pub fn from_dword(val: u32) -> Self {
        Self::new(
            ((val >> 28) & 0x0F) as u8,
            ((val >> 24) & 0x0F) as u8,
            ((val >> 20) & 0x0F) as u8,
            ((val >> 16) & 0x0F) as u8,
            ((val >> 12) & 0x0F) as u8,
            ((val >> 8) & 0x0F) as u8,
            ((val >> 4) & 0x0F) as u8,
            (val & 0x0F) as u8,
        )
    }
}

impl From<zymatica_zspar::Concept8D> for Concept8DState {
    fn from(c: zymatica_zspar::Concept8D) -> Self {
        Self::from_zspar(c)
    }
}

impl From<Concept8DState> for zymatica_zspar::Concept8D {
    fn from(c: Concept8DState) -> Self {
        c.to_zspar()
    }
}

/// Z-SPAR Semantic Codeword (12 nibbles = 8 data + 4 parity over GF(16))
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SemanticCodeword8D {
    pub state: Concept8DState,
    pub parity: [u8; 4],
}

impl SemanticCodeword8D {
    /// Generator matrix G for Systematic RS(12,8) over GF(16)
    /// Parity symbols P_j = sum_{i=0..7} G[j][i] * data[i]
    pub fn encode(state: Concept8DState) -> Self {
        let d = state.to_symbols();
        let mut p = [0u8; 4];

        // Systematic Cauchy-Reed-Solomon generator coefficients over GF(16)
        for j in 0..4 {
            let mut sum = 0u8;
            let root = GF16::EXP[j + 1]; // alpha^(j+1)
            for (i, &val) in d.iter().enumerate() {
                let weight = GF16::power(root, i + 1);
                let term = GF16::mul(val, weight);
                sum = GF16::add(sum, term);
            }
            p[j] = sum;
        }

        Self { state, parity: p }
    }

    /// Convert 12-nibble codeword into 6 packed bytes
    pub fn to_bytes(&self) -> [u8; 6] {
        let syms = self.to_symbols();
        let mut bytes = [0u8; 6];
        for i in 0..6 {
            bytes[i] = (syms[i * 2] << 4) | (syms[i * 2 + 1] & 0x0F);
        }
        bytes
    }

    /// Parse 12-nibble codeword from 6 packed bytes
    pub fn from_bytes(bytes: &[u8; 6]) -> Self {
        let mut syms = [0u8; 12];
        for (i, &b) in bytes.iter().enumerate() {
            syms[i * 2] = (b >> 4) & 0x0F;
            syms[i * 2 + 1] = b & 0x0F;
        }
        let mut data = [0u8; 8];
        data.copy_from_slice(&syms[0..8]);
        let mut parity = [0u8; 4];
        parity.copy_from_slice(&syms[8..12]);

        Self {
            state: Concept8DState::from_symbols(&data),
            parity,
        }
    }

    pub fn to_symbols(&self) -> [u8; 12] {
        let d = self.state.to_symbols();
        [
            d[0],
            d[1],
            d[2],
            d[3],
            d[4],
            d[5],
            d[6],
            d[7],
            self.parity[0],
            self.parity[1],
            self.parity[2],
            self.parity[3],
        ]
    }
}

/// Result of Semantic Syndrome Analysis and Auto-Correction
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SemanticSyndromeResult {
    /// Meaning is 100% intact across models; 0 drift detected
    ExactMatch,
    /// Semantic drift detected and fully corrected locally via GF(16) RS(12,8) decoding
    Corrected {
        drifted_axes: Vec<usize>,
        corrected_state: Concept8DState,
    },
    /// Severe semantic divergence (>2 axes); requires targeted 3-byte Semantic Repair Chirp
    UncorrectableDivergence {
        syndrome: [u8; 4],
        drifted_axes_mask: u8,
    },
}

/// Z-SPAR Core Decoder & Semantic Repair Engine
pub struct ZSparEngine;

impl ZSparEngine {
    /// Evaluate semantic syndrome for a received concept against parity
    pub fn verify_and_repair(
        reconstructed_state: Concept8DState,
        expected_parity: [u8; 4],
    ) -> SemanticSyndromeResult {
        let d = reconstructed_state.to_symbols();
        let mut syndromes = [0u8; 4];
        let mut all_zero = true;

        // Compute 4 syndromes: S_j = P_expected[j] ^ P_computed[j]
        for j in 0..4 {
            let mut sum = 0u8;
            let root = GF16::EXP[j + 1];
            for (i, &val) in d.iter().enumerate() {
                let weight = GF16::power(root, i + 1);
                let term = GF16::mul(val, weight);
                sum = GF16::add(sum, term);
            }
            let syndrome = GF16::add(expected_parity[j], sum);
            syndromes[j] = syndrome;
            if syndrome != 0 {
                all_zero = false;
            }
        }

        if all_zero {
            return SemanticSyndromeResult::ExactMatch;
        }

        // Attempt Single-Axis Semantic Correction (1-symbol error)
        for target_axis in 0..8 {
            let mut candidate_error = None;
            let mut consistent = true;

            for j in 0..4 {
                let root = GF16::EXP[j + 1];
                let weight = GF16::power(root, target_axis + 1);
                // S_j = weight * error  =>  error = S_j / weight
                if let Ok(err) = GF16::div(syndromes[j], weight) {
                    if let Some(prev_err) = candidate_error {
                        if prev_err != err {
                            consistent = false;
                            break;
                        }
                    } else {
                        candidate_error = Some(err);
                    }
                } else {
                    consistent = false;
                    break;
                }
            }

            if consistent && candidate_error.is_some() && candidate_error != Some(0) {
                let err = candidate_error.unwrap();
                let mut corrected_symbols = d;
                corrected_symbols[target_axis] = GF16::add(corrected_symbols[target_axis], err);

                return SemanticSyndromeResult::Corrected {
                    drifted_axes: vec![target_axis],
                    corrected_state: Concept8DState::from_symbols(&corrected_symbols),
                };
            }
        }

        // Attempt Two-Axis Semantic Correction (2-symbol errors)
        for i1 in 0..8 {
            for i2 in (i1 + 1)..8 {
                // Solve 2x2 system using S_0 and S_1:
                // w1_0 * e1 ^ w2_0 * e2 = S_0
                // w1_1 * e1 ^ w2_1 * e2 = S_1
                let r0 = GF16::EXP[1];
                let r1 = GF16::EXP[2];
                let a11 = GF16::power(r0, i1 + 1);
                let a12 = GF16::power(r0, i2 + 1);
                let a21 = GF16::power(r1, i1 + 1);
                let a22 = GF16::power(r1, i2 + 1);

                // Determinant det = a11*a22 ^ a12*a21
                let det = GF16::add(GF16::mul(a11, a22), GF16::mul(a12, a21));
                if det == 0 {
                    continue;
                }

                // e1 = (a22*S0 ^ a12*S1) / det
                // e2 = (a11*S1 ^ a21*S0) / det
                let num1 = GF16::add(GF16::mul(a22, syndromes[0]), GF16::mul(a12, syndromes[1]));
                let num2 = GF16::add(GF16::mul(a11, syndromes[1]), GF16::mul(a21, syndromes[0]));

                if let (Ok(e1), Ok(e2)) = (GF16::div(num1, det), GF16::div(num2, det)) {
                    // Check if (e1, e2) satisfy S_2 and S_3
                    let r2 = GF16::EXP[3];
                    let r3 = GF16::EXP[4];
                    let check_s2 = GF16::add(
                        GF16::mul(GF16::power(r2, i1 + 1), e1),
                        GF16::mul(GF16::power(r2, i2 + 1), e2),
                    );
                    let check_s3 = GF16::add(
                        GF16::mul(GF16::power(r3, i1 + 1), e1),
                        GF16::mul(GF16::power(r3, i2 + 1), e2),
                    );

                    if check_s2 == syndromes[2] && check_s3 == syndromes[3] {
                        let mut corrected_symbols = d;
                        corrected_symbols[i1] = GF16::add(corrected_symbols[i1], e1);
                        corrected_symbols[i2] = GF16::add(corrected_symbols[i2], e2);

                        return SemanticSyndromeResult::Corrected {
                            drifted_axes: vec![i1, i2],
                            corrected_state: Concept8DState::from_symbols(&corrected_symbols),
                        };
                    }
                }
            }
        }

        // Divergence exceeds local 2-axis correction capacity; emit targeted repair chirp metadata
        let mut mask = 0u8;
        for (i, &s) in syndromes.iter().enumerate() {
            if s != 0 {
                mask |= 1 << i;
            }
        }

        SemanticSyndromeResult::UncorrectableDivergence {
            syndrome: syndromes,
            drifted_axes_mask: mask,
        }
    }

    /// Build a 3-byte Semantic Repair Chirp for micro-band RF transmission
    pub fn build_repair_chirp(trajectory_id: u8, mask: u8, syndrome_tag: u8) -> [u8; 3] {
        [trajectory_id, mask, syndrome_tag]
    }
}

/// Layer 2: Semantic Invariant Guard protecting critical numbers, entities, and tool targets
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct SemanticInvariantGuard {
    pub entity_hashes: HashMap<String, u32>,
    pub critical_numerical_values: HashMap<String, i64>,
    pub negation_bit: bool,
}

impl SemanticInvariantGuard {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn protect_entity(&mut self, entity_type: &str, entity_val: &str) {
        let mut hasher = 0x811c9dc5u32;
        for &b in entity_val.as_bytes() {
            hasher ^= b as u32;
            hasher = hasher.wrapping_mul(0x01000193);
        }
        self.entity_hashes.insert(entity_type.to_string(), hasher);
    }

    pub fn protect_number(&mut self, parameter_name: &str, value: i64) {
        self.critical_numerical_values
            .insert(parameter_name.to_string(), value);
    }

    pub fn set_negation(&mut self, is_negated: bool) {
        self.negation_bit = is_negated;
    }

    pub fn verify_invariants(&self, other: &Self) -> bool {
        if self.negation_bit != other.negation_bit {
            return false;
        }
        for (k, v) in &self.critical_numerical_values {
            if other.critical_numerical_values.get(k) != Some(v) {
                return false;
            }
        }
        for (k, v) in &self.entity_hashes {
            if other.entity_hashes.get(k) != Some(v) {
                return false;
            }
        }
        true
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_gf16_arithmetic_properties() {
        assert_eq!(GF16::add(5, 5), 0);
        assert_eq!(GF16::mul(0, 7), 0);
        assert_eq!(GF16::mul(1, 9), 9);

        // Multiplicative inverse property: a * (1/a) == 1
        for a in 1..16 {
            let inv = GF16::div(1, a).expect("Valid GF(16) inverse");
            assert_eq!(GF16::mul(a, inv), 1);
        }
    }

    #[test]
    fn test_z_spar_lossless_roundtrip_no_drift() {
        let original = Concept8DState::new(1, 2, 8, 4, 12, 1, 6, 15);
        let codeword = SemanticCodeword8D::encode(original);

        let bytes = codeword.to_bytes();
        assert_eq!(bytes.len(), 6);

        let decoded = SemanticCodeword8D::from_bytes(&bytes);
        assert_eq!(decoded.state, original);

        let verify_result = ZSparEngine::verify_and_repair(decoded.state, decoded.parity);
        assert_eq!(verify_result, SemanticSyndromeResult::ExactMatch);
    }

    #[test]
    fn test_z_spar_single_axis_model_semantic_drift_correction() {
        // Node A sends: [Domain=1, Subdomain=4, OP=8 (CLOSE), Modality=15 (MANDATORY), Strength=10, Polarity=1, Time=2, Epistemic=14]
        let true_intent = Concept8DState::new(1, 4, 8, 15, 10, 1, 2, 14);
        let codeword = SemanticCodeword8D::encode(true_intent);

        // Node B's local LLM (e.g. SmolLM2) drifts on Operation: OP=8 (CLOSE) -> OP=3 (REDUCE)
        let drifted_intent = Concept8DState::new(1, 4, 3, 15, 10, 1, 2, 14);

        // Z-SPAR computes syndrome and repairs OP back to 8 without retransmission!
        let result = ZSparEngine::verify_and_repair(drifted_intent, codeword.parity);

        match result {
            SemanticSyndromeResult::Corrected {
                drifted_axes,
                corrected_state,
            } => {
                assert_eq!(drifted_axes, vec![2]); // Axis 2 is Operation (OP)
                assert_eq!(corrected_state.operation, 8); // Repaired to CLOSE
                assert_eq!(corrected_state, true_intent);
            }
            _ => panic!("Expected single-axis correction"),
        }
    }

    #[test]
    fn test_z_spar_two_axis_cross_model_semantic_drift_correction() {
        // Node A sends: OP=8 (CLOSE), Modality=15 (MANDATORY)
        let true_intent = Concept8DState::new(3, 7, 8, 15, 9, 2, 4, 12);
        let codeword = SemanticCodeword8D::encode(true_intent);

        // Node B's local LLM drifts on 2 axes: OP (8->2) and Modality (15->5)
        let drifted_intent = Concept8DState::new(3, 7, 2, 5, 9, 2, 4, 12);

        let result = ZSparEngine::verify_and_repair(drifted_intent, codeword.parity);

        match result {
            SemanticSyndromeResult::Corrected {
                drifted_axes,
                corrected_state,
            } => {
                assert_eq!(drifted_axes, vec![2, 3]); // OP and Modality
                assert_eq!(corrected_state.operation, 8);
                assert_eq!(corrected_state.modality, 15);
                assert_eq!(corrected_state, true_intent);
            }
            _ => panic!("Expected dual-axis correction"),
        }
    }

    #[test]
    fn test_z_spar_semantic_invariant_guard() {
        let mut guard_tx = SemanticInvariantGuard::new();
        guard_tx.protect_entity("valve_id", "VALVE_7");
        guard_tx.protect_number("pressure_psi", 50);
        guard_tx.set_negation(false);

        let mut guard_rx_valid = SemanticInvariantGuard::new();
        guard_rx_valid.protect_entity("valve_id", "VALVE_7");
        guard_rx_valid.protect_number("pressure_psi", 50);
        guard_rx_valid.set_negation(false);

        let mut guard_rx_tampered = SemanticInvariantGuard::new();
        guard_rx_tampered.protect_entity("valve_id", "VALVE_9"); // Tampered entity
        guard_rx_tampered.protect_number("pressure_psi", 50);
        guard_rx_tampered.set_negation(false);

        assert!(guard_tx.verify_invariants(&guard_rx_valid));
        assert!(!guard_tx.verify_invariants(&guard_rx_tampered));
    }
}
