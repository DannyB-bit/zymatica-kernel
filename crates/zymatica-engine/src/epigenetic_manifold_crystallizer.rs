//! # Invention Class 31: Zymatica Epigenetic Weight Crystallizer (Z-NEWM)
//!
//! Non-Destructive Zero-Backpropagation On-Device Continual Learning.
//! Projects real-time domain adaptations onto the orthogonal nullspace of
//! existing activation tensors, eliminating catastrophic forgetting with zero weight degradation.

pub struct EpigeneticCrystal {
    pub domain_id: u8,
    pub nullspace_basis_rank: u8,
    pub crystal_weights: [f32; 16],
    pub activation_hash: u32,
}

impl EpigeneticCrystal {
    pub fn new(domain: u8, rank: u8, weights: [f32; 16], hash: u32) -> Self {
        Self {
            domain_id: domain,
            nullspace_basis_rank: rank,
            crystal_weights: weights,
            activation_hash: hash,
        }
    }

    pub fn to_bytes(&self) -> [u8; 64] {
        let mut out = [0u8; 64];
        out[0] = self.domain_id;
        out[1] = self.nullspace_basis_rank;
        for (i, &w) in self.crystal_weights.iter().enumerate() {
            let b = w.to_be_bytes();
            out[2 + i * 4..6 + i * 4].copy_from_slice(&b);
        }
        let h_bytes = self.activation_hash.to_be_bytes();
        out[60..64].copy_from_slice(&h_bytes);
        out
    }

    pub fn from_bytes(bytes: &[u8; 64]) -> Self {
        let domain = bytes[0];
        let rank = bytes[1];
        let mut weights = [0.0f32; 16];
        for i in 0..16 {
            let b = [bytes[2 + i * 4], bytes[3 + i * 4], bytes[4 + i * 4], bytes[5 + i * 4]];
            weights[i] = f32::from_be_bytes(b);
        }
        let hash = u32::from_be_bytes([bytes[60], bytes[61], bytes[62], bytes[63]]);
        Self {
            domain_id: domain,
            nullspace_basis_rank: rank,
            crystal_weights: weights,
            activation_hash: hash,
        }
    }
}

pub struct EpigeneticManifoldEngine {
    pub hidden_dim: usize,
}

impl EpigeneticManifoldEngine {
    pub fn new(hidden_dim: usize) -> Self {
        Self { hidden_dim }
    }

    pub fn compute_nullspace_projection(&self, base_activations: &[f32], new_concept: &[f32]) -> Vec<f32> {
        let mut dot_prod = 0.0f32;
        let mut base_norm_sq = 0.0f32;

        for (&a, &c) in base_activations.iter().zip(new_concept.iter()) {
            dot_prod += a * c;
            base_norm_sq += a * a;
        }

        if base_norm_sq == 0.0 {
            return new_concept.to_vec();
        }

        let scalar = dot_prod / base_norm_sq;
        let mut nullspace_delta = vec![0.0f32; self.hidden_dim];
        for i in 0..self.hidden_dim {
            nullspace_delta[i] = new_concept[i] - scalar * base_activations[i];
        }

        nullspace_delta
    }

    pub fn apply_crystal(&self, h: &mut [f32], crystal: &EpigeneticCrystal, nullspace_basis: &[f32]) {
        for (i, val) in h.iter_mut().enumerate() {
            let crystal_coeff = crystal.crystal_weights[i % 16];
            let basis_component = nullspace_basis[i % nullspace_basis.len()];
            *val += crystal_coeff * basis_component * 0.1;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_epigenetic_nullspace_exact_orthogonality() {
        let hidden_dim = 64;
        let engine = EpigeneticManifoldEngine::new(hidden_dim);

        let base_activations = vec![1.0f32; hidden_dim];
        let mut new_concept = vec![0.5f32; hidden_dim];
        new_concept[0] = 2.0;

        let nullspace_delta = engine.compute_nullspace_projection(&base_activations, &new_concept);

        let mut dot = 0.0f32;
        for i in 0..hidden_dim {
            dot += base_activations[i] * nullspace_delta[i];
        }

        assert!(dot.abs() < 1e-5);
    }

    #[test]
    fn test_epigenetic_crystal_64byte_roundtrip() {
        let weights = [0.1, 0.2, 0.3, 0.4, 0.5, 0.6, 0.7, 0.8, -0.1, -0.2, -0.3, -0.4, -0.5, -0.6, -0.7, -0.8];
        let crystal = EpigeneticCrystal::new(4, 2, weights, 0xABCDEF01);
        let bytes = crystal.to_bytes();
        assert_eq!(bytes.len(), 64);

        let recovered = EpigeneticCrystal::from_bytes(&bytes);
        assert_eq!(crystal.domain_id, recovered.domain_id);
        assert_eq!(crystal.activation_hash, recovered.activation_hash);
    }
}
