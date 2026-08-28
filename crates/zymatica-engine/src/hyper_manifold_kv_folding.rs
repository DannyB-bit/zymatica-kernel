//! # Invention Class 29: Zymatica Hyper-Manifold KV Folding (Hyper-KV) - Production Hardened

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
}
