//! # Invention Class 29: Zymatica Hyper-Manifold KV Folding (Hyper-KV) - Production Hardened

/// Hyper-Manifold KV Knot representing a continuous trajectory segment in KV embedding space
#[derive(Debug, Clone, PartialEq)]
pub struct HyperKvKnotLUT {
    pub base_coord: [f32; 6],
    pub delta_tangent: [f32; 6],
    pub lut_index: u8,
    pub span_tokens: u8,
}

impl HyperKvKnotLUT {
    pub const PHASE_LUT: [f32; 16] = [
        0.0000, 0.3826, 0.7071, 0.9238, 1.0000, 0.9238, 0.7071, 0.3826,
        0.0000, -0.3826, -0.7071, -0.9238, -1.0000, -0.9238, -0.7071, -0.3826,
    ];

    #[inline(always)]
    pub fn evaluate_lut(&self, t: usize, head_dim: usize) -> Vec<f32> {
        let t_norm = if self.span_tokens > 1 { t as f32 / (self.span_tokens - 1) as f32 } else { 0.0 };
        let mut out = vec![0.0f32; head_dim];
        
        let lut_val = Self::PHASE_LUT[(self.lut_index as usize + t) % 16];

        for i in 0..head_dim {
            let axis = i % 6;
            let base_val = self.base_coord[axis] + self.delta_tangent[axis] * t_norm;
            out[i] = base_val + lut_val * 0.05;
        }
        out
    }

    /// Compress an unrolled sequence of KV vectors [tokens x head_dim] into a parametric Knot LUT
    pub fn compress_block(kv_vectors: &[f32], tokens: usize, head_dim: usize) -> Self {
        assert_eq!(kv_vectors.len(), tokens * head_dim);
        let mut base_coord = [0.0f32; 6];
        let mut end_coord = [0.0f32; 6];

        // First token projection
        for i in 0..head_dim {
            base_coord[i % 6] += kv_vectors[i];
        }
        for a in 0..6 {
            let count = (head_dim + 5 - a) / 6;
            if count > 0 {
                base_coord[a] /= count as f32;
            }
        }

        // Last token projection
        let last_offset = (tokens.saturating_sub(1)) * head_dim;
        for i in 0..head_dim {
            end_coord[i % 6] += kv_vectors[last_offset + i];
        }
        for a in 0..6 {
            let count = (head_dim + 5 - a) / 6;
            if count > 0 {
                end_coord[a] /= count as f32;
            }
        }

        let mut delta_tangent = [0.0f32; 6];
        for a in 0..6 {
            delta_tangent[a] = end_coord[a] - base_coord[a];
        }

        Self {
            base_coord,
            delta_tangent,
            lut_index: (tokens % 16) as u8,
            span_tokens: tokens.min(255) as u8,
        }
    }

    /// Reconstruct an entire KV block [tokens x head_dim] from this Knot LUT
    pub fn reconstruct_block(&self, tokens: usize, head_dim: usize) -> Vec<f32> {
        let mut block = Vec::with_capacity(tokens * head_dim);
        for t in 0..tokens {
            let vec_t = self.evaluate_lut(t, head_dim);
            block.extend_from_slice(&vec_t);
        }
        block
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_hyper_kv_compression_reconstruction_parity() {
        let tokens = 16;
        let head_dim = 64;
        let mut original_kv = vec![0.0f32; tokens * head_dim];
        for t in 0..tokens {
            for d in 0..head_dim {
                original_kv[t * head_dim + d] = (t as f32 * 0.1) + ((d % 6) as f32 * 0.5);
            }
        }

        let knot = HyperKvKnotLUT::compress_block(&original_kv, tokens, head_dim);
        assert_eq!(knot.span_tokens as usize, tokens);

        let reconstructed = knot.reconstruct_block(tokens, head_dim);
        assert_eq!(reconstructed.len(), original_kv.len());

        // Verify smooth trajectory continuity
        let mut mse = 0.0f32;
        for i in 0..original_kv.len() {
            let diff = original_kv[i] - reconstructed[i];
            mse += diff * diff;
        }
        mse /= original_kv.len() as f32;
        assert!(mse < 0.05, "Reconstruction MSE should be tight: {}", mse);
    }
}
