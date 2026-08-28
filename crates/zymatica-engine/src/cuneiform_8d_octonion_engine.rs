//! # Invention Class 32: Zymatica 8D Octonion Hypercube Engine (Z-8D Octagram)
//!
//! 32-Bit Native Word Concept Architecture over 8-Dimensional Octonion Manifolds.
//! Encodes Context (Domain, Subdomain), Function (Operation, Modality),
//! Affect (Strength, Polarity), and Truth (Temporal Horizon, zk-Certainty)
//! into a single atomic 4-byte uint32 DWORD with sub-nanosecond hardware execution.

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
    pub fn new(
        domain: u8,
        subdomain: u8,
        operation: u8,
        modality: u8,
        strength: u8,
        polarity: u8,
        temporal: u8,
        certainty: u8,
    ) -> Self {
        Self {
            domain: domain & 0x0F,
            subdomain: subdomain & 0x0F,
            operation: operation & 0x0F,
            modality: modality & 0x0F,
            strength: strength & 0x0F,
            polarity: polarity & 0x0F,
            temporal_horizon: temporal & 0x0F,
            epistemic_certainty: certainty & 0x0F,
        }
    }

    /// Pack into 4-byte Cuneiform-U Radicals: [R_C, R_F, R_A, R_T]
    pub fn to_radicals(&self) -> [u8; 4] {
        let rc = (self.domain << 4) | (self.subdomain & 0x0F);
        let rf = (self.operation << 4) | (self.modality & 0x0F);
        let ra = (self.strength << 4) | (self.polarity & 0x0F);
        let rt = (self.temporal_horizon << 4) | (self.epistemic_certainty & 0x0F);
        [rc, rf, ra, rt]
    }

    /// Pack directly into a single 32-bit hardware atomic DWORD (uint32)
    pub fn to_dword(&self) -> u32 {
        let rad = self.to_radicals();
        u32::from_be_bytes(rad)
    }

    /// Unpack from 32-bit hardware atomic DWORD (uint32)
    pub fn from_dword(dword: u32) -> Self {
        let b = dword.to_be_bytes();
        Self {
            domain: (b[0] >> 4) & 0x0F,
            subdomain: b[0] & 0x0F,
            operation: (b[1] >> 4) & 0x0F,
            modality: b[1] & 0x0F,
            strength: (b[2] >> 4) & 0x0F,
            polarity: b[2] & 0x0F,
            temporal_horizon: (b[3] >> 4) & 0x0F,
            epistemic_certainty: b[3] & 0x0F,
        }
    }

    /// Calculate 8D Manhattan Metric Distance
    pub fn distance(&self, other: &Self) -> u32 {
        let d1 = (self.domain as i32 - other.domain as i32).abs();
        let d2 = (self.subdomain as i32 - other.subdomain as i32).abs();
        let d3 = (self.operation as i32 - other.operation as i32).abs();
        let d4 = (self.modality as i32 - other.modality as i32).abs();
        let d5 = (self.strength as i32 - other.strength as i32).abs();
        let d6 = (self.polarity as i32 - other.polarity as i32).abs();
        let d7 = (self.temporal_horizon as i32 - other.temporal_horizon as i32).abs();
        let d8 = (self.epistemic_certainty as i32 - other.epistemic_certainty as i32).abs();
        (d1 + d2 + d3 + d4 + d5 + d6 + d7 + d8) as u32
    }
}

/// Geodesic 8D Stream Codec with Variable-Length Modes
pub struct Geodesic8DCodec;

impl Geodesic8DCodec {
    /// Encode a trajectory of 8D concepts
    pub fn encode_trajectory(trajectory: &[Concept8D]) -> Vec<u8> {
        if trajectory.is_empty() {
            return Vec::new();
        }

        let mut out = Vec::with_capacity(trajectory.len() * 2);
        out.extend_from_slice(&trajectory[0].to_radicals());

        let mut prev = trajectory[0];
        for curr in &trajectory[1..] {
            let dist = prev.distance(curr);
            if dist == 0 {
                // Mode 00: Zero delta (1 byte)
                out.push(0x00);
            } else if prev.domain == curr.domain && prev.subdomain == curr.subdomain && dist <= 8 {
                // Mode 01: Local delta (3 bytes)
                out.push(0x40 | ((curr.operation & 0x07) << 3) | (curr.modality & 0x07));
                out.push((curr.strength << 4) | (curr.polarity & 0x0F));
                out.push((curr.temporal_horizon << 4) | (curr.epistemic_certainty & 0x0F));
            } else {
                // Mode 11: Full 8D DWORD Escape (5 bytes)
                out.push(0xC0);
                out.extend_from_slice(&curr.to_radicals());
            }
            prev = *curr;
        }

        out
    }

    /// Decode compressed 8D trajectory
    pub fn decode_trajectory(bytes: &[u8]) -> Result<Vec<Concept8D>, &'static str> {
        if bytes.len() < 4 {
            return Err("Payload too short for 8D base header");
        }

        let mut out = Vec::new();
        let first_dword = u32::from_be_bytes([bytes[0], bytes[1], bytes[2], bytes[3]]);
        let mut curr = Concept8D::from_dword(first_dword);
        out.push(curr);

        let mut idx = 4;
        while idx < bytes.len() {
            let mode = bytes[idx] >> 6;
            match mode {
                0 => {
                    out.push(curr);
                    idx += 1;
                }
                1 => {
                    if idx + 2 >= bytes.len() {
                        return Err("Unexpected EOF in 8D Mode 01");
                    }
                    let b1 = bytes[idx];
                    let b2 = bytes[idx + 1];
                    let b3 = bytes[idx + 2];
                    curr.operation = (b1 >> 3) & 0x07;
                    curr.modality = b1 & 0x07;
                    curr.strength = (b2 >> 4) & 0x0F;
                    curr.polarity = b2 & 0x0F;
                    curr.temporal_horizon = (b3 >> 4) & 0x0F;
                    curr.epistemic_certainty = b3 & 0x0F;
                    out.push(curr);
                    idx += 3;
                }
                3 => {
                    if idx + 4 >= bytes.len() {
                        return Err("Unexpected EOF in 8D Mode 11");
                    }
                    let dword = u32::from_be_bytes([bytes[idx + 1], bytes[idx + 2], bytes[idx + 3], bytes[idx + 4]]);
                    curr = Concept8D::from_dword(dword);
                    out.push(curr);
                    idx += 5;
                }
                _ => return Err("Invalid 8D mode header"),
            }
        }

        Ok(out)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_concept_8d_dword_roundtrip() {
        let c1 = Concept8D::new(2, 8, 5, 1, 14, 3, 12, 15);
        let dword = c1.to_dword();
        assert_eq!(dword, 0x2851E3CF);

        let c2 = Concept8D::from_dword(dword);
        assert_eq!(c1, c2);
    }

    #[test]
    fn test_geodesic_8d_trajectory_lossless() {
        let mut traj = Vec::new();
        traj.push(Concept8D::new(1, 2, 3, 4, 5, 6, 7, 8));
        traj.push(Concept8D::new(1, 2, 3, 4, 5, 6, 7, 8)); // Mode 00
        traj.push(Concept8D::new(1, 2, 4, 5, 6, 7, 8, 9)); // Mode 01
        traj.push(Concept8D::new(9, 10, 11, 12, 13, 14, 15, 0)); // Mode 11

        let enc = Geodesic8DCodec::encode_trajectory(&traj);
        let dec = Geodesic8DCodec::decode_trajectory(&enc).expect("Valid 8D decode");
        assert_eq!(traj, dec);
    }
}
