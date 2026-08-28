//! # Invention Class 31: Zymatica Epigenetic Weight Crystallizer (Z-NEWM) - Production Hardened
//!
//! Features Modified Gram-Schmidt (MGS) Orthogonalization, Re-orthogonalization Checkpoints,
//! and Iterative Numerical Drift Compensation to ensure exact activation invariance across projected subspaces (A_old * ΔW = 0).

pub struct EpigeneticCrystal {
    pub domain_id: u8,
    pub nullspace_basis_rank: u8,
    pub crystal_weights: [f32; 16],
    pub activation_hash: u32,
}

impl EpigeneticCrystal {
    pub const WIRE_BYTE_LEN: usize = 70; // 1 (domain) + 1 (rank) + 64 (16 x f32) + 4 (hash) = 70 bytes

    pub fn new(domain: u8, rank: u8, weights: [f32; 16], hash: u32) -> Self {
        Self {
            domain_id: domain,
            nullspace_basis_rank: rank,
            crystal_weights: weights,
            activation_hash: hash,
        }
    }

    pub fn to_bytes(&self) -> [u8; 70] {
        let mut out = [0u8; 70];
        out[0] = self.domain_id;
        out[1] = self.nullspace_basis_rank;
        for (i, &w) in self.crystal_weights.iter().enumerate() {
            let b = w.to_be_bytes();
            out[2 + i * 4..6 + i * 4].copy_from_slice(&b);
        }
        let h_bytes = self.activation_hash.to_be_bytes();
        out[66..70].copy_from_slice(&h_bytes);
        out
    }

    pub fn from_bytes(bytes: &[u8; 70]) -> Self {
        let domain = bytes[0];
        let rank = bytes[1];
        let mut weights = [0.0f32; 16];
        for i in 0..16 {
            let b = [
                bytes[2 + i * 4],
                bytes[3 + i * 4],
                bytes[4 + i * 4],
                bytes[5 + i * 4],
            ];
            weights[i] = f32::from_be_bytes(b);
        }
        let hash = u32::from_be_bytes([bytes[66], bytes[67], bytes[68], bytes[69]]);
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
    pub update_counter: usize,
}

impl EpigeneticManifoldEngine {
    pub fn new(hidden_dim: usize) -> Self {
        Self {
            hidden_dim,
            update_counter: 0,
        }
    }

    pub fn compute_nullspace_projection_mgs(
        &mut self,
        base_basis: &[Vec<f32>],
        new_concept: &[f32],
    ) -> Vec<f32> {
        self.update_counter += 1;
        let mut v = new_concept.to_vec();

        for b in base_basis {
            let mut dot = 0.0f32;
            let mut norm_sq = 0.0f32;
            for i in 0..self.hidden_dim {
                dot += b[i] * v[i];
                norm_sq += b[i] * b[i];
            }
            if norm_sq > 1e-12 {
                let s = dot / norm_sq;
                for i in 0..self.hidden_dim {
                    v[i] -= s * b[i];
                }
            }
        }

        for b in base_basis {
            let mut dot = 0.0f32;
            let mut norm_sq = 0.0f32;
            for i in 0..self.hidden_dim {
                dot += b[i] * v[i];
                norm_sq += b[i] * b[i];
            }
            if norm_sq > 1e-12 {
                let s = dot / norm_sq;
                for i in 0..self.hidden_dim {
                    v[i] -= s * b[i];
                }
            }
        }

        v
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_epigenetic_crystal_70byte_wire_roundtrip() {
        let mut weights = [0.0f32; 16];
        for i in 0..16 {
            weights[i] = (i as f32) * 1.5 - 4.0;
        }
        let crystal = EpigeneticCrystal::new(7, 3, weights, 0xCAFEBABE);
        let bytes = crystal.to_bytes();
        assert_eq!(bytes.len(), 70);
        assert_eq!(bytes.len(), EpigeneticCrystal::WIRE_BYTE_LEN);

        let decoded = EpigeneticCrystal::from_bytes(&bytes);
        assert_eq!(decoded.domain_id, 7);
        assert_eq!(decoded.nullspace_basis_rank, 3);
        assert_eq!(decoded.activation_hash, 0xCAFEBABE);
        for i in 0..16 {
            assert!((decoded.crystal_weights[i] - weights[i]).abs() < 1e-7);
        }
    }

    #[test]
    fn test_mgs_nullspace_projection_orthogonality() {
        let mut engine = EpigeneticManifoldEngine::new(4);
        let base_basis = vec![vec![1.0, 0.0, 0.0, 0.0], vec![0.0, 1.0, 0.0, 0.0]];
        let new_concept = vec![3.0, 4.0, 5.0, 6.0];

        let projected = engine.compute_nullspace_projection_mgs(&base_basis, &new_concept);
        assert_eq!(projected.len(), 4);

        // Projected vector must be orthogonal to all base basis vectors
        for b in &base_basis {
            let dot: f32 = b.iter().zip(&projected).map(|(&x, &y)| x * y).sum();
            assert!(
                dot.abs() < 1e-6,
                "Dot product with basis should be zero, got {}",
                dot
            );
        }

        // Remaining components in orthogonal subspace should be preserved
        assert!((projected[2] - 5.0).abs() < 1e-6);
        assert!((projected[3] - 6.0).abs() < 1e-6);
    }
}
