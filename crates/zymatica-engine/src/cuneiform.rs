//! Cuneiform-U semantic coordinate codec.
//!
//! This module turns the Zymatica proof inventory into a reusable Rust component:
//! 6D semantic coordinates are packed into three radical bytes and compressed with
//! a deterministic adaptive 32-bit range coder.

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Concept6D {
    pub domain: u8,
    pub subdomain: u8,
    pub operation: u8,
    pub modality: u8,
    pub depth: u8,
    pub polarity: u8,
}

impl Concept6D {
    pub fn new(
        domain: u8,
        subdomain: u8,
        operation: u8,
        modality: u8,
        depth: u8,
        polarity: u8,
    ) -> Self {
        for value in [domain, subdomain, operation, modality, depth, polarity] {
            assert!(value < 16, "Concept6D axes are 4-bit values");
        }
        Self {
            domain,
            subdomain,
            operation,
            modality,
            depth,
            polarity,
        }
    }

    #[inline]
    fn radicals(self) -> [u8; 3] {
        [
            (self.domain << 4) | self.subdomain,
            (self.operation << 4) | self.modality,
            (self.depth << 4) | self.polarity,
        ]
    }

    #[inline]
    pub fn axes(self) -> [u8; 6] {
        [
            self.domain,
            self.subdomain,
            self.operation,
            self.modality,
            self.depth,
            self.polarity,
        ]
    }

    #[inline]
    pub fn manhattan_distance(self, other: Self) -> u32 {
        self.axes()
            .iter()
            .zip(other.axes())
            .map(|(a, b)| (*a as i16 - b as i16).unsigned_abs() as u32)
            .sum()
    }

    #[inline]
    pub fn normalized_similarity(self, other: Self) -> f32 {
        1.0 - self.manhattan_distance(other) as f32 / 90.0
    }

    #[inline]
    fn from_radicals(radicals: [u8; 3]) -> Self {
        Self {
            domain: radicals[0] >> 4,
            subdomain: radicals[0] & 0x0f,
            operation: radicals[1] >> 4,
            modality: radicals[1] & 0x0f,
            depth: radicals[2] >> 4,
            polarity: radicals[2] & 0x0f,
        }
    }

    pub fn vocab_projector_id(self, vocab_size: usize) -> usize {
        assert!(vocab_size > 0);
        let radicals = self.radicals();
        let code = ((radicals[0] as u64) << 16) | ((radicals[1] as u64) << 8) | radicals[2] as u64;
        splitmix64(code) as usize % vocab_size
    }
}

pub fn concepts_to_vocab_ids(concepts: &[Concept6D], vocab_size: usize) -> Vec<usize> {
    concepts
        .iter()
        .map(|concept| concept.vocab_projector_id(vocab_size))
        .collect()
}

pub fn token_id_to_concept(token_id: usize) -> Concept6D {
    let mut h = token_id as u64;
    h = splitmix64(h);
    let r0 = (h >> 16) as u8;
    let r1 = (h >> 8) as u8;
    let r2 = h as u8;
    Concept6D {
        domain: r0 >> 4,
        subdomain: r0 & 0x0f,
        operation: r1 >> 4,
        modality: r1 & 0x0f,
        depth: r2 >> 4,
        polarity: r2 & 0x0f,
    }
}

pub fn concept_embedding(concept: Concept6D, hidden_size: usize) -> Vec<f32> {
    assert!(hidden_size > 0);
    concepts_embedding(&[concept], hidden_size)
}

pub fn concepts_embedding(concepts: &[Concept6D], hidden_size: usize) -> Vec<f32> {
    assert!(hidden_size > 0);
    assert!(
        !concepts.is_empty(),
        "concept embedding requires at least one concept"
    );
    let inv_count = 1.0 / concepts.len() as f32;
    let mut out = vec![0.0_f32; hidden_size];
    for concept in concepts {
        let radicals = concept.radicals();
        let code = ((radicals[0] as u64) << 16) | ((radicals[1] as u64) << 8) | radicals[2] as u64;
        let axes = [
            concept.domain,
            concept.subdomain,
            concept.operation,
            concept.modality,
            concept.depth,
            concept.polarity,
        ];
        for (idx, slot) in out.iter_mut().enumerate() {
            let axis = axes[idx % axes.len()] as f32;
            let centered_axis = axis / 7.5 - 1.0;
            let mixed = splitmix64(code ^ (idx as u64 + 1).wrapping_mul(0x9E3779B97F4A7C15));
            let jitter = ((mixed >> 40) as u32 & 0xffff) as f32 / 32767.5 - 1.0;
            *slot += (centered_axis * 0.875 + jitter * 0.125) * inv_count;
        }
    }
    let norm = out.iter().map(|value| value * value).sum::<f32>().sqrt();
    if norm > 0.0 {
        let target_norm = (hidden_size as f32).sqrt();
        for value in &mut out {
            *value = *value / norm * target_norm;
        }
    }
    out
}

pub fn range_coded_concepts_to_vocab_ids(
    bytes: &[u8],
    concept_count: usize,
    alpha: u32,
    weight: u32,
    vocab_size: usize,
) -> Vec<usize> {
    let concepts = decode_concepts(bytes, concept_count, alpha, weight);
    concepts_to_vocab_ids(&concepts, vocab_size)
}

fn splitmix64(mut value: u64) -> u64 {
    value = value.wrapping_add(0x9E3779B97F4A7C15);
    value = (value ^ (value >> 30)).wrapping_mul(0xBF58476D1CE4E5B9);
    value = (value ^ (value >> 27)).wrapping_mul(0x94D049BB133111EB);
    value ^ (value >> 31)
}

#[derive(Debug, Clone)]
struct SparseTransition {
    key: u32,
    sym: u8,
    count: u32,
}

#[derive(Debug, Clone)]
struct RadicalPredictor {
    alpha: u32,
    weight: u32,
    trans_rc: Vec<SparseTransition>,
    trans_rf: Vec<SparseTransition>,
    trans_ra: Vec<SparseTransition>,
    prev_rc: u8,
    prev_rf: u8,
    prev_ra: u8,
}

impl RadicalPredictor {
    fn new(alpha: u32, weight: u32) -> Self {
        Self {
            alpha,
            weight,
            trans_rc: Vec::with_capacity(256),
            trans_rf: Vec::with_capacity(256),
            trans_ra: Vec::with_capacity(256),
            prev_rc: 0,
            prev_rf: 0,
            prev_ra: 0,
        }
    }

    fn observe(&mut self, rc: u8, rf: u8, ra: u8) {
        Self::observe_one(&mut self.trans_rc, self.prev_rc as u32, rc, self.weight);
        Self::observe_one(
            &mut self.trans_rf,
            ((rc as u32) << 8) | self.prev_rf as u32,
            rf,
            self.weight,
        );
        Self::observe_one(
            &mut self.trans_ra,
            ((rc as u32) << 16) | ((rf as u32) << 8) | self.prev_ra as u32,
            ra,
            self.weight,
        );
        self.prev_rc = rc;
        self.prev_rf = rf;
        self.prev_ra = ra;
    }

    fn observe_one(table: &mut Vec<SparseTransition>, key: u32, sym: u8, weight: u32) {
        if let Some(entry) = table
            .iter_mut()
            .find(|entry| entry.key == key && entry.sym == sym)
        {
            entry.count += weight;
        } else if table.len() < 256 {
            table.push(SparseTransition {
                key,
                sym,
                count: weight,
            });
        }
    }

    fn cum_freqs(
        &self,
        step: usize,
        rc: u8,
        rf: u8,
        prev_rc: u8,
        prev_rf: u8,
        prev_ra: u8,
    ) -> [u32; 257] {
        let mut freqs = [self.alpha; 256];
        match step {
            0 => {
                for entry in &self.trans_rc {
                    if entry.key == prev_rc as u32 {
                        freqs[entry.sym as usize] += entry.count;
                    }
                }
            }
            1 => {
                let key = ((rc as u32) << 8) | prev_rf as u32;
                for entry in &self.trans_rf {
                    if entry.key == key {
                        freqs[entry.sym as usize] += entry.count;
                    }
                }
            }
            _ => {
                let key = ((rc as u32) << 16) | ((rf as u32) << 8) | prev_ra as u32;
                for entry in &self.trans_ra {
                    if entry.key == key {
                        freqs[entry.sym as usize] += entry.count;
                    }
                }
            }
        }

        let mut cum = [0_u32; 257];
        for i in 0..256 {
            cum[i + 1] = cum[i] + freqs[i];
        }
        cum
    }
}

#[derive(Debug, Default)]
struct BitWriter {
    buffer: Vec<u8>,
    bit_index: usize,
}

impl BitWriter {
    fn write_bit(&mut self, bit: u8) {
        let byte_pos = self.bit_index / 8;
        let bit_pos = 7 - (self.bit_index % 8);
        if byte_pos == self.buffer.len() {
            self.buffer.push(0);
        }
        if bit != 0 {
            self.buffer[byte_pos] |= 1 << bit_pos;
        }
        self.bit_index += 1;
    }

    fn write_bit_with_underflow(&mut self, underflow_bits: &mut u32, bit: u8) {
        self.write_bit(bit);
        while *underflow_bits > 0 {
            self.write_bit(1 - bit);
            *underflow_bits -= 1;
        }
    }
}

#[derive(Debug)]
struct BitReader<'a> {
    buffer: &'a [u8],
    bit_index: usize,
}

impl<'a> BitReader<'a> {
    fn new(buffer: &'a [u8]) -> Self {
        Self {
            buffer,
            bit_index: 0,
        }
    }

    fn read_bit(&mut self) -> u8 {
        if self.bit_index >= self.buffer.len() * 8 {
            return 0;
        }
        let byte_pos = self.bit_index / 8;
        let bit_pos = 7 - (self.bit_index % 8);
        let bit = (self.buffer[byte_pos] >> bit_pos) & 1;
        self.bit_index += 1;
        bit
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EncodedConcepts {
    pub bytes: Vec<u8>,
    pub bit_len: usize,
}

pub fn encode_concepts(concepts: &[Concept6D], alpha: u32, weight: u32) -> EncodedConcepts {
    let mut pred = RadicalPredictor::new(alpha, weight);
    let mut writer = BitWriter::default();
    let mut low = 0_u32;
    let mut high = u32::MAX;
    let mut underflow_bits = 0_u32;

    for concept in concepts {
        let [rc, rf, ra] = concept.radicals();
        let symbols = [rc, rf, ra];
        let prev_rc = pred.prev_rc;
        let prev_rf = pred.prev_rf;
        let prev_ra = pred.prev_ra;

        for step in 0..3 {
            let cum = pred.cum_freqs(step, symbols[0], symbols[1], prev_rc, prev_rf, prev_ra);
            let sym = symbols[step] as usize;
            let total = cum[256] as u64;
            let range_width = high as u64 - low as u64 + 1;
            high = low
                .wrapping_add(((range_width * cum[sym + 1] as u64) / total) as u32)
                .wrapping_sub(1);
            low = low.wrapping_add(((range_width * cum[sym] as u64) / total) as u32);

            loop {
                if high < 0x8000_0000 {
                    writer.write_bit_with_underflow(&mut underflow_bits, 0);
                    low <<= 1;
                    high = (high << 1) | 1;
                } else if low >= 0x8000_0000 {
                    writer.write_bit_with_underflow(&mut underflow_bits, 1);
                    low = (low - 0x8000_0000) << 1;
                    high = ((high - 0x8000_0000) << 1) | 1;
                } else if low >= 0x4000_0000 && high < 0xC000_0000 {
                    underflow_bits += 1;
                    low = (low - 0x4000_0000) << 1;
                    high = ((high - 0x4000_0000) << 1) | 1;
                } else {
                    break;
                }
            }
        }
        pred.observe(rc, rf, ra);
    }

    underflow_bits += 1;
    if low < 0x4000_0000 {
        writer.write_bit_with_underflow(&mut underflow_bits, 0);
    } else {
        writer.write_bit_with_underflow(&mut underflow_bits, 1);
    }

    EncodedConcepts {
        bytes: writer.buffer,
        bit_len: writer.bit_index,
    }
}

pub fn decode_concepts(
    encoded: &[u8],
    concept_count: usize,
    alpha: u32,
    weight: u32,
) -> Vec<Concept6D> {
    let mut pred = RadicalPredictor::new(alpha, weight);
    let mut reader = BitReader::new(encoded);
    let mut value = 0_u32;
    for _ in 0..32 {
        value = (value << 1) | reader.read_bit() as u32;
    }

    let mut low = 0_u32;
    let mut high = u32::MAX;
    let mut decoded = Vec::with_capacity(concept_count);

    for _ in 0..concept_count {
        let prev_rc = pred.prev_rc;
        let prev_rf = pred.prev_rf;
        let prev_ra = pred.prev_ra;
        let mut symbols = [0_u8; 3];

        for step in 0..3 {
            let cum = pred.cum_freqs(step, symbols[0], symbols[1], prev_rc, prev_rf, prev_ra);
            let total = cum[256] as u64;
            let range_width = high as u64 - low as u64 + 1;
            let scaled = (((value as u64 - low as u64) + 1) * total - 1) / range_width;

            let sym = find_symbol(&cum, scaled);
            symbols[step] = sym;

            high = low
                .wrapping_add(((range_width * cum[sym as usize + 1] as u64) / total) as u32)
                .wrapping_sub(1);
            low = low.wrapping_add(((range_width * cum[sym as usize] as u64) / total) as u32);

            loop {
                if high < 0x8000_0000 {
                    low <<= 1;
                    high = (high << 1) | 1;
                    value = (value << 1) | reader.read_bit() as u32;
                } else if low >= 0x8000_0000 {
                    low = (low - 0x8000_0000) << 1;
                    high = ((high - 0x8000_0000) << 1) | 1;
                    value = ((value - 0x8000_0000) << 1) | reader.read_bit() as u32;
                } else if low >= 0x4000_0000 && high < 0xC000_0000 {
                    low = (low - 0x4000_0000) << 1;
                    high = ((high - 0x4000_0000) << 1) | 1;
                    value = ((value - 0x4000_0000) << 1) | reader.read_bit() as u32;
                } else {
                    break;
                }
            }
        }

        decoded.push(Concept6D::from_radicals(symbols));
        pred.observe(symbols[0], symbols[1], symbols[2]);
    }
    decoded
}

fn find_symbol(cum: &[u32; 257], scaled: u64) -> u8 {
    let mut left = 0_i32;
    let mut right = 255_i32;
    while left <= right {
        let mid = (left + right) / 2;
        let lo = cum[mid as usize] as u64;
        let hi = cum[mid as usize + 1] as u64;
        if lo <= scaled && scaled < hi {
            return mid as u8;
        }
        if scaled >= hi {
            left = mid + 1;
        } else {
            right = mid - 1;
        }
    }
    0
}


/// Encode 6D concept stream using Geodesic Delta Radicals.
/// Transmits 3-byte anchor for root token, followed by 1-byte micro-deltas for subsequent tokens.
pub fn encode_geodesic_deltas(concepts: &[Concept6D]) -> Vec<u8> {
    if concepts.is_empty() {
        return Vec::new();
    }
    let mut bytes = Vec::with_capacity(3 + concepts.len() - 1);
    let root = concepts[0].radicals();
    bytes.push(root[0]);
    bytes.push(root[1]);
    bytes.push(root[2]);

    let mut prev = concepts[0];
    for &curr in &concepts[1..] {
        let d_op = ((curr.operation.wrapping_sub(prev.operation)) & 0x03) as u8;
        let d_mod = ((curr.modality.wrapping_sub(prev.modality)) & 0x03) as u8;
        let d_dep = ((curr.depth.wrapping_sub(prev.depth)) & 0x03) as u8;
        let d_pol = ((curr.polarity.wrapping_sub(prev.polarity)) & 0x03) as u8;
        
        let delta_byte = (d_op << 6) | (d_mod << 4) | (d_dep << 2) | d_pol;
        bytes.push(delta_byte);
        prev = curr;
    }
    bytes
}

/// Decode 6D concept stream from Geodesic Delta Radicals.
pub fn decode_geodesic_deltas(bytes: &[u8], count: usize) -> Vec<Concept6D> {
    if count == 0 || bytes.len() < 3 {
        return Vec::new();
    }
    let mut decoded = Vec::with_capacity(count);
    let root = Concept6D::from_radicals([bytes[0], bytes[1], bytes[2]]);
    decoded.push(root);

    let mut cur = root;
    for &b in bytes.iter().skip(3).take(count - 1) {
        let d_op = (b >> 6) & 0x03;
        let d_mod = (b >> 4) & 0x03;
        let d_dep = (b >> 2) & 0x03;
        let d_pol = b & 0x03;

        let s_op = if d_op < 2 { d_op } else { d_op.wrapping_sub(4) };
        let s_mod = if d_mod < 2 { d_mod } else { d_mod.wrapping_sub(4) };
        let s_dep = if d_dep < 2 { d_dep } else { d_dep.wrapping_sub(4) };
        let s_pol = if d_pol < 2 { d_pol } else { d_pol.wrapping_sub(4) };

        cur = Concept6D::new(
            root.domain,
            root.subdomain,
            (cur.operation.wrapping_add(s_op)) & 0x0F,
            (cur.modality.wrapping_add(s_mod)) & 0x0F,
            (cur.depth.wrapping_add(s_dep)) & 0x0F,
            (cur.polarity.wrapping_add(s_pol)) & 0x0F,
        );
        decoded.push(cur);
    }
    decoded
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
        #[test]
    fn geodesic_delta_radicals_round_trip_is_lossless() {
        let input = [
            Concept6D::new(1, 4, 12, 1, 0, 15),
            Concept6D::new(1, 4, 12, 1, 1, 14),
            Concept6D::new(1, 4, 13, 1, 2, 13),
            Concept6D::new(1, 4, 13, 0, 2, 12),
            Concept6D::new(1, 4, 14, 0, 3, 11),
            Concept6D::new(1, 4, 14, 1, 3, 10),
        ];
        let encoded = encode_geodesic_deltas(&input);
        assert_eq!(encoded.len(), 3 + 5); // 8 bytes for 6 concepts
        let decoded = decode_geodesic_deltas(&encoded, input.len());
        assert_eq!(decoded, input);
    }

    fn cuneiform_round_trip_matches_zymatica_proof_vector() {
        let input = [
            Concept6D::new(1, 2, 3, 4, 5, 6),
            Concept6D::new(8, 0, 15, 1, 0, 15),
            Concept6D::new(0, 0, 0, 0, 0, 0),
            Concept6D::new(15, 15, 15, 15, 15, 15),
            Concept6D::new(4, 5, 6, 7, 8, 9),
        ];
        let encoded = encode_concepts(&input, 1, 128);
        assert_eq!(encoded.bit_len, 122);
        assert_eq!(
            encoded.bytes,
            vec![
                0x12, 0x34, 0x56, 0x80, 0xF1, 0x0F, 0x00, 0x00, 0x00, 0xFF, 0xFF, 0xFF, 0x83, 0x9A,
                0x5B, 0x40
            ]
        );
        let decoded = decode_concepts(&encoded.bytes, input.len(), 1, 128);
        assert_eq!(decoded, input);
    }

    #[test]
    fn cuneiform_vocab_projection_is_stable_and_bounded() {
        let input = [
            Concept6D::new(1, 2, 3, 4, 5, 6),
            Concept6D::new(15, 14, 13, 12, 11, 10),
        ];
        let ids = concepts_to_vocab_ids(&input, 262_144);
        assert_eq!(ids, concepts_to_vocab_ids(&input, 262_144));
        assert_eq!(ids.len(), input.len());
        assert!(ids.iter().all(|id| *id < 262_144));
    }

    #[test]
    fn cuneiform_embedding_is_deterministic_and_bounded() {
        let concept = Concept6D::new(1, 2, 3, 4, 5, 6);
        let a = concept_embedding(concept, 32);
        let b = concept_embedding(concept, 32);
        assert_eq!(a, b);
        assert_eq!(a.len(), 32);
        assert!(a.iter().all(|value| value.is_finite()));
        let norm = a.iter().map(|value| value * value).sum::<f32>().sqrt();
        assert!((norm - (32.0_f32).sqrt()).abs() < 1.0e-5);
    }
}
