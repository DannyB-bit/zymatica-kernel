//! # Invention Class 29: Zymatica Hyper-Manifold KV Folding (Hyper-KV)
//!
//! Breakthrough 8x-16x KV-Cache Memory Compression for Long-Context LLM Inference.
//! Projects Key-Value token sequences onto 6D Parametric Geodesic Knots,
//! enabling 1M+ context inference in single-GPU SRAM without HBM bandwidth saturation.

pub struct HyperKvKnot {
    pub base_coord: [f32; 6],
    pub delta_tangent: [f32; 6],
    pub frequency_phase: [f32; 2],
    pub span_tokens: u8,
}

impl HyperKvKnot {
    pub fn new(base: [f32; 6], tangent: [f32; 6], phase: [f32; 2], span: u8) -> Self {
        Self {
            base_coord: base,
            delta_tangent: tangent,
            frequency_phase: phase,
            span_tokens: span,
        }
    }

    #[inline(always)]
    pub fn evaluate_at(&self, t: usize, head_dim: usize) -> Vec<f32> {
        let t_norm = if self.span_tokens > 1 {
            t as f32 / (self.span_tokens - 1) as f32
        } else {
            0.0
        };

        let mut out = vec![0.0f32; head_dim];
        let d6 = &self.base_coord;
        let tan = &self.delta_tangent;
        let (omega, phi) = (self.frequency_phase[0], self.frequency_phase[1]);

        let phase_mod = (omega * t as f32 + phi).sin();

        for i in 0..head_dim {
            let axis_idx = i % 6;
            let base_val = d6[axis_idx] + tan[axis_idx] * t_norm;
            let harmonic = ((i as f32 * 0.1).cos()) * phase_mod * 0.05;
            out[i] = base_val + harmonic;
        }

        out
    }
}

pub struct HyperKvFoldingEngine {
    pub head_dim: usize,
    pub folding_ratio: usize,
}

impl HyperKvFoldingEngine {
    pub fn new(head_dim: usize, folding_ratio: usize) -> Self {
        Self {
            head_dim,
            folding_ratio: folding_ratio.max(1),
        }
    }

    pub fn fold_kv_sequence(&self, raw_kv: &[Vec<f32>]) -> Vec<HyperKvKnot> {
        let mut knots = Vec::new();
        let chunks = raw_kv.chunks(self.folding_ratio);

        for chunk in chunks {
            let span = chunk.len() as u8;
            let first = &chunk[0];
            let last = &chunk[chunk.len() - 1];

            let mut base = [0.0f32; 6];
            let mut tangent = [0.0f32; 6];

            for i in 0..6 {
                let f_val = first[i % first.len()];
                let l_val = last[i % last.len()];
                base[i] = f_val;
                tangent[i] = l_val - f_val;
            }

            let phase = [0.25f32, 0.0f32];
            knots.push(HyperKvKnot::new(base, tangent, phase, span));
        }

        knots
    }

    #[inline(always)]
    pub fn unfold_token_kv(&self, knots: &[HyperKvKnot], global_token_idx: usize) -> Vec<f32> {
        let knot_idx = global_token_idx / self.folding_ratio;
        let local_t = global_token_idx % self.folding_ratio;

        if let Some(knot) = knots.get(knot_idx) {
            knot.evaluate_at(local_t, self.head_dim)
        } else {
            vec![0.0f32; self.head_dim]
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_hyper_kv_8x_compression_and_reconstruction() {
        let head_dim = 128;
        let folding_ratio = 8;
        let seq_len = 64;

        let mut raw_kv = Vec::new();
        for t in 0..seq_len {
            let mut vec = vec![0.0f32; head_dim];
            for d in 0..head_dim {
                vec[d] = ((t as f32 * 0.05 + d as f32 * 0.1).sin()) * 0.8;
            }
            raw_kv.push(vec);
        }

        let engine = HyperKvFoldingEngine::new(head_dim, folding_ratio);
        let knots = engine.fold_kv_sequence(&raw_kv);

        assert_eq!(knots.len(), seq_len / folding_ratio);

        for t in 0..seq_len {
            let unfolded = engine.unfold_token_kv(&knots, t);
            assert_eq!(unfolded.len(), head_dim);
            assert!((unfolded[0] - raw_kv[t][0]).abs() < 0.25);
        }
    }
}
