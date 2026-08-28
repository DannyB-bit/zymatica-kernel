//! # Invention Class 32: Zymatica 8D Octonion Hypercube Engine (Z-8D) - Production Hardened

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Concept8D {
    pub domain: u8,
    pub subdomain: u8,
    pub operation: u8,
    pub modality: u8,
    pub strength: u8,
    pub polarity: u8,
    pub temporal_horizon: u8,
    pub epistemic_certainty: u8,
}

impl Concept8D {
    pub fn new(d: u8, sub: u8, op: u8, mod_: u8, st: u8, pol: u8, temp: u8, cert: u8) -> Self {
        Self {
            domain: d & 0x0F,
            subdomain: sub & 0x0F,
            operation: op & 0x0F,
            modality: mod_ & 0x0F,
            strength: st & 0x0F,
            polarity: pol & 0x0F,
            temporal_horizon: temp & 0x0F,
            epistemic_certainty: cert & 0x0F,
        }
    }

    pub fn to_dword(&self) -> u32 {
        let rc = (self.domain << 4) | (self.subdomain & 0x0F);
        let rf = (self.operation << 4) | (self.modality & 0x0F);
        let ra = (self.strength << 4) | (self.polarity & 0x0F);
        let rt = (self.temporal_horizon << 4) | (self.epistemic_certainty & 0x0F);
        ((rc as u32) << 24) | ((rf as u32) << 16) | ((ra as u32) << 8) | (rt as u32)
    }

    pub fn from_dword(dword: u32) -> Self {
        Self {
            domain: ((dword >> 28) & 0x0F) as u8,
            subdomain: ((dword >> 24) & 0x0F) as u8,
            operation: ((dword >> 20) & 0x0F) as u8,
            modality: ((dword >> 16) & 0x0F) as u8,
            strength: ((dword >> 12) & 0x0F) as u8,
            polarity: ((dword >> 8) & 0x0F) as u8,
            temporal_horizon: ((dword >> 4) & 0x0F) as u8,
            epistemic_certainty: (dword & 0x0F) as u8,
        }
    }
}

pub struct Geodesic8DCodecHardened;

impl Geodesic8DCodecHardened {
    pub const KEYFRAME_INTERVAL: usize = 16;

    pub fn encode_trajectory(trajectory: &[Concept8D]) -> Vec<u8> {
        if trajectory.is_empty() { return Vec::new(); }
        let mut out = Vec::with_capacity(trajectory.len() * 3);
        
        let mut prev = trajectory[0];
        for (i, curr) in trajectory.iter().enumerate() {
            if i % Self::KEYFRAME_INTERVAL == 0 {
                out.push(0xC0);
                out.extend_from_slice(&curr.to_dword().to_be_bytes());
            } else if *curr == prev {
                out.push(0x00);
            } else {
                out.push(0x40 | ((curr.operation & 0x07) << 3) | (curr.modality & 0x07));
                out.push((curr.strength << 4) | (curr.polarity & 0x0F));
                out.push((curr.temporal_horizon << 4) | (curr.epistemic_certainty & 0x0F));
            }
            prev = *curr;
        }
        out
    }
}
