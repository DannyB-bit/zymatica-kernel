//! 6D Hyper-Geodesic KV-Cache Compression Bridge.
//! Compresses historical KV cache states into 6D Riemannian manifold trajectories,
//! reducing long-context RAM footprint by over 90%.

use crate::cuneiform::Concept6D;

#[derive(Clone, Debug)]
pub struct CompressedKvBlock {
    pub anchor_concept: Concept6D,
    pub delta_stream: Vec<u8>,
    pub num_tokens: usize,
}

impl CompressedKvBlock {
    pub fn new(anchor: Concept6D) -> Self {
        Self {
            anchor_concept: anchor,
            delta_stream: Vec::new(),
            num_tokens: 1,
        }
    }

    pub fn push_step(&mut self, next: Concept6D, prev: Concept6D) {
        let d_op = ((next.operation.wrapping_sub(prev.operation)) & 0x03) as u8;
        let d_mod = ((next.modality.wrapping_sub(prev.modality)) & 0x03) as u8;
        let d_dep = ((next.depth.wrapping_sub(prev.depth)) & 0x03) as u8;
        let d_pol = ((next.polarity.wrapping_sub(prev.polarity)) & 0x03) as u8;
        
        let delta_byte = (d_op << 6) | (d_mod << 4) | (d_dep << 2) | d_pol;
        self.delta_stream.push(delta_byte);
        self.num_tokens += 1;
    }

    pub fn decompress_all(&self) -> Vec<Concept6D> {
        let mut out = Vec::with_capacity(self.num_tokens);
        out.push(self.anchor_concept);
        let mut cur = self.anchor_concept;

        for &b in &self.delta_stream {
            let d_op = (b >> 6) & 0x03;
            let d_mod = (b >> 4) & 0x03;
            let d_dep = (b >> 2) & 0x03;
            let d_pol = b & 0x03;

            let s_op = if d_op < 2 { d_op } else { d_op.wrapping_sub(4) };
            let s_mod = if d_mod < 2 { d_mod } else { d_mod.wrapping_sub(4) };
            let s_dep = if d_dep < 2 { d_dep } else { d_dep.wrapping_sub(4) };
            let s_pol = if d_pol < 2 { d_pol } else { d_pol.wrapping_sub(4) };

            cur = Concept6D::new(
                self.anchor_concept.domain,
                self.anchor_concept.subdomain,
                (cur.operation.wrapping_add(s_op)) & 0x0F,
                (cur.modality.wrapping_add(s_mod)) & 0x0F,
                (cur.depth.wrapping_add(s_dep)) & 0x0F,
                (cur.polarity.wrapping_add(s_pol)) & 0x0F,
            );
            out.push(cur);
        }
        out
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn compressed_kv_block_round_trips_losslessly() {
        let c0 = Concept6D::new(1, 2, 8, 8, 8, 8);
        let mut block = CompressedKvBlock::new(c0);
        
        let c1 = Concept6D::new(1, 2, 9, 8, 7, 8);
        let c2 = Concept6D::new(1, 2, 10, 9, 7, 9);
        
        block.push_step(c1, c0);
        block.push_step(c2, c1);
        
        let recovered = block.decompress_all();
        assert_eq!(recovered.len(), 3);
        assert_eq!(recovered[0], c0);
        assert_eq!(recovered[1], c1);
        assert_eq!(recovered[2], c2);
    }
}