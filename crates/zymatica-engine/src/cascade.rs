use anyhow::{Context, Result, bail};
use flate2::Compression;
use flate2::read::ZlibDecoder;
use flate2::write::ZlibEncoder;
use std::collections::HashMap;
use std::io::{Read, Write};

pub trait CompressionStage {
    fn compress(&self, data: &[u8]) -> Result<Vec<u8>>;
    fn decompress(&self, data: &[u8]) -> Result<Vec<u8>>;
}

// ============================================================================
// Level 1: Tokenizer (text -> token IDs)
// ============================================================================
pub struct Level1Tokenizer;

impl CompressionStage for Level1Tokenizer {
    fn compress(&self, data: &[u8]) -> Result<Vec<u8>> {
        // Map raw bytes directly to Unicode code points 0..255 (Latin-1 block).
        // This is always lossless and never fails, regardless of whether data is valid UTF-8.
        let text: String = data.iter().map(|&b| b as char).collect();

        // Tokenize losslessly by extracting consecutive alphanumeric and non-alphanumeric chunks
        let mut words = Vec::new();
        let mut current = String::new();
        let mut is_word = true;
        for c in text.chars() {
            let char_is_word = c.is_alphanumeric();
            let char_len = c.len_utf8();
            if current.is_empty() {
                is_word = char_is_word;
                current.push(c);
            } else if is_word == char_is_word && current.len() + char_len <= 255 {
                current.push(c);
            } else {
                words.push(current.clone());
                current.clear();
                is_word = char_is_word;
                current.push(c);
            }
        }
        if !current.is_empty() {
            words.push(current);
        }

        let mut vocab = Vec::new();
        let mut vocab_map = HashMap::new();
        for w in &words {
            if !vocab_map.contains_key(w.as_str()) {
                vocab_map.insert(w.as_str(), vocab.len());
                vocab.push(w.as_str());
            }
        }

        if vocab.len() > 65535 {
            bail!("Vocabulary size exceeds 16-bit capacity");
        }

        let mut out = Vec::new();
        // Pack vocab size: 2 bytes Big Endian
        out.extend_from_slice(&(vocab.len() as u16).to_be_bytes());

        // Pack words: length (1 byte) + word bytes
        for word in &vocab {
            let wb = word.as_bytes();
            if wb.len() > 255 {
                bail!("Word length exceeds 8-bit capacity");
            }
            out.push(wb.len() as u8);
            out.extend_from_slice(wb);
        }

        // Pack token IDs: use 1-byte if vocab size <= 256, else 2-byte BE indices
        let use_two_bytes = vocab.len() > 256;
        for w in &words {
            let id = vocab_map.get(w.as_str()).copied().unwrap();
            if use_two_bytes {
                out.extend_from_slice(&(id as u16).to_be_bytes());
            } else {
                out.push(id as u8);
            }
        }

        Ok(out)
    }

    fn decompress(&self, data: &[u8]) -> Result<Vec<u8>> {
        if data.len() < 2 {
            bail!("Input too short for tokenizer decompression");
        }
        let mut offset = 0;
        let vocab_size = u16::from_be_bytes([data[0], data[1]]) as usize;
        offset += 2;

        let mut vocab = Vec::with_capacity(vocab_size);
        for _ in 0..vocab_size {
            if offset >= data.len() {
                bail!("Unexpected EOF reading vocabulary");
            }
            let wlen = data[offset] as usize;
            offset += 1;
            if offset + wlen > data.len() {
                bail!("Unexpected EOF reading word content");
            }
            let word = std::str::from_utf8(&data[offset..offset + wlen])
                .context("invalid UTF-8 in vocab word")?;
            offset += wlen;
            vocab.push(word);
        }

        let use_two_bytes = vocab_size > 256;
        let mut tokens = Vec::new();
        while offset < data.len() {
            let tid = if use_two_bytes {
                if offset + 2 > data.len() {
                    bail!("Unexpected EOF reading 2-byte token ID");
                }
                let id = u16::from_be_bytes([data[offset], data[offset + 1]]) as usize;
                offset += 2;
                id
            } else {
                let id = data[offset] as usize;
                offset += 1;
                id
            };
            if tid >= vocab.len() {
                bail!("Token ID out of vocabulary bounds");
            }
            tokens.push(vocab[tid]);
        }

        let decoded_string = tokens.join("");
        // Convert Latin-1 code point characters back to raw bytes.
        let decoded_bytes: Vec<u8> = decoded_string.chars().map(|c| c as u8).collect();
        Ok(decoded_bytes)
    }
}

// ============================================================================
// Level 2: Prefix-Suffix Deduplication
// ============================================================================
pub struct Level2PrefixSuffix;

impl CompressionStage for Level2PrefixSuffix {
    fn compress(&self, data: &[u8]) -> Result<Vec<u8>> {
        let chunk_size = 4;
        let mut out = Vec::new();
        // Header: original length (2 bytes BE)
        if data.len() > 65535 {
            bail!("Input size exceeds 16-bit capacity");
        }
        out.extend_from_slice(&(data.len() as u16).to_be_bytes());

        let mut seen: HashMap<Vec<u8>, u16> = HashMap::new();
        let chunks = data.chunks(chunk_size);

        for chunk in chunks {
            if chunk.len() == chunk_size && seen.contains_key(chunk) {
                let index = seen.get(chunk).copied().unwrap();
                out.push(0xFF);
                out.extend_from_slice(&index.to_be_bytes());
            } else {
                if chunk.len() == chunk_size {
                    let index = seen.len() as u16;
                    seen.insert(chunk.to_vec(), index);
                }
                out.push(chunk.len() as u8);
                out.extend_from_slice(chunk);
            }
        }

        Ok(out)
    }

    fn decompress(&self, data: &[u8]) -> Result<Vec<u8>> {
        if data.len() < 2 {
            bail!("Input too short for prefix-suffix decompression");
        }
        let orig_len = u16::from_be_bytes([data[0], data[1]]) as usize;
        let mut offset = 2;

        let mut seen: HashMap<u16, Vec<u8>> = HashMap::new();
        let mut result = Vec::new();
        let mut index: u16 = 0;

        while offset < data.len() {
            let marker = data[offset];
            offset += 1;

            if marker == 0xFF {
                if offset + 2 > data.len() {
                    bail!("Unexpected EOF reading reference index");
                }
                let ref_val = u16::from_be_bytes([data[offset], data[offset + 1]]);
                offset += 2;
                let chunk = seen.get(&ref_val).context("missing reference chunk")?;
                result.extend_from_slice(chunk);
            } else {
                let chunk_len = marker as usize;
                if offset + chunk_len > data.len() {
                    bail!("Unexpected EOF reading literal chunk");
                }
                let chunk = &data[offset..offset + chunk_len];
                offset += chunk_len;
                if chunk_len == 4 {
                    seen.insert(index, chunk.to_vec());
                    index += 1;
                }
                result.extend_from_slice(chunk);
            }
        }

        result.truncate(orig_len);
        Ok(result)
    }
}

// ============================================================================
// Level 3: Delta Encoding
// ============================================================================
pub struct Level3Delta;

impl CompressionStage for Level3Delta {
    fn compress(&self, data: &[u8]) -> Result<Vec<u8>> {
        let mut out = Vec::new();
        if data.is_empty() {
            out.extend_from_slice(&[0, 0]);
            return Ok(out);
        }
        if data.len() > 65535 {
            bail!("Input size exceeds 16-bit capacity");
        }
        out.extend_from_slice(&(data.len() as u16).to_be_bytes());
        out.push(data[0]);

        for i in 1..data.len() {
            let delta = ((data[i] as i16 - data[i - 1] as i16) & 0xFF) as u8;
            out.push(delta);
        }

        Ok(out)
    }

    fn decompress(&self, data: &[u8]) -> Result<Vec<u8>> {
        if data.len() < 2 {
            bail!("Input too short for delta decompression");
        }
        let orig_len = u16::from_be_bytes([data[0], data[1]]) as usize;
        if orig_len == 0 {
            return Ok(Vec::new());
        }
        if data.len() < 2 + orig_len {
            bail!("Delta input size is smaller than original length declared in header");
        }

        let mut result = Vec::with_capacity(orig_len);
        result.push(data[2]);

        for &delta in &data[3..2 + orig_len] {
            let val = ((result.last().copied().unwrap() as u16 + delta as u16) & 0xFF) as u8;
            result.push(val);
        }

        Ok(result)
    }
}

// ============================================================================
// Level 4: Run-Length Encoding
// ============================================================================
pub struct Level4Rle;

impl CompressionStage for Level4Rle {
    fn compress(&self, data: &[u8]) -> Result<Vec<u8>> {
        let mut out = Vec::new();
        if data.is_empty() {
            out.extend_from_slice(&[0, 0]);
            return Ok(out);
        }
        if data.len() > 65535 {
            bail!("Input size exceeds 16-bit capacity");
        }
        out.extend_from_slice(&(data.len() as u16).to_be_bytes());

        let mut i = 0;
        while i < data.len() {
            let mut count = 1;
            while i + count < data.len() && data[i + count] == data[i] && count < 255 {
                count += 1;
            }

            if count >= 3 {
                out.push(0xFF);
                out.push(count as u8);
                out.push(data[i]);
            } else {
                for j in 0..count {
                    let b = data[i + j];
                    if b == 0xFF {
                        out.extend_from_slice(&[0xFF, 0x01, 0xFF]);
                    } else {
                        out.push(b);
                    }
                }
            }
            i += count;
        }

        Ok(out)
    }

    fn decompress(&self, data: &[u8]) -> Result<Vec<u8>> {
        if data.len() < 2 {
            bail!("Input too short for RLE decompression");
        }
        let orig_len = u16::from_be_bytes([data[0], data[1]]) as usize;
        if orig_len == 0 {
            return Ok(Vec::new());
        }

        let mut result = Vec::new();
        let mut i = 2;

        while i < data.len() && result.len() < orig_len {
            if data[i] == 0xFF && i + 2 < data.len() {
                let count = data[i + 1] as usize;
                let val = data[i + 2];
                result.extend(std::iter::repeat_n(val, count));
                i += 3;
            } else {
                result.push(data[i]);
                i += 1;
            }
        }

        result.truncate(orig_len);
        Ok(result)
    }
}

// ============================================================================
// Level 5: Byte-Frequency Reordering
// ============================================================================
pub struct Level5FreqReorder;

impl CompressionStage for Level5FreqReorder {
    fn compress(&self, data: &[u8]) -> Result<Vec<u8>> {
        if data.is_empty() {
            return Ok(Vec::new());
        }

        let mut freq = HashMap::new();
        for &b in data {
            *freq.entry(b).or_insert(0) += 1;
        }

        let mut sorted_bytes: Vec<u8> = freq.keys().copied().collect();
        // Sort descending by frequency, break ties with byte value ascending
        sorted_bytes.sort_by(|a, b| {
            let fa = freq.get(a).unwrap();
            let fb = freq.get(b).unwrap();
            fb.cmp(fa).then_with(|| a.cmp(b))
        });

        let mut mapping = HashMap::new();
        for (i, &b) in sorted_bytes.iter().enumerate() {
            mapping.insert(b, i as u8);
        }

        let mut out = Vec::new();
        // Table size is encoded as size - 1 to support up to 256 bytes in a single u8
        out.push((sorted_bytes.len() - 1) as u8);
        out.extend_from_slice(&sorted_bytes);

        if data.len() > 65535 {
            bail!("Input size exceeds 16-bit capacity");
        }
        out.extend_from_slice(&(data.len() as u16).to_be_bytes());

        for &b in data {
            let mapped = mapping.get(&b).copied().unwrap();
            out.push(mapped);
        }

        Ok(out)
    }

    fn decompress(&self, data: &[u8]) -> Result<Vec<u8>> {
        if data.is_empty() {
            return Ok(Vec::new());
        }
        let mut offset = 0;
        let table_size = (data[offset] as usize) + 1;
        offset += 1;

        if offset + table_size + 2 > data.len() {
            bail!("Unexpected EOF reading reordering mapping table");
        }
        let table = &data[offset..offset + table_size];
        offset += table_size;

        let orig_len = u16::from_be_bytes([data[offset], data[offset + 1]]) as usize;
        offset += 2;

        if offset + orig_len > data.len() {
            bail!("Unexpected EOF reading reordered data body");
        }

        let mut result = Vec::with_capacity(orig_len);
        for i in 0..orig_len {
            let idx = data[offset + i] as usize;
            if idx >= table.len() {
                bail!("Reordered index out of table bounds");
            }
            result.push(table[idx]);
        }

        Ok(result)
    }
}

// ============================================================================
// Level 6: zlib Deflate
// ============================================================================
pub struct Level6Zlib;

impl CompressionStage for Level6Zlib {
    fn compress(&self, data: &[u8]) -> Result<Vec<u8>> {
        let mut encoder = ZlibEncoder::new(Vec::new(), Compression::best());
        encoder.write_all(data)?;
        let compressed = encoder.finish()?;

        let mut out = Vec::new();
        if data.len() > 65535 {
            bail!("Input size exceeds 16-bit capacity");
        }
        out.extend_from_slice(&(data.len() as u16).to_be_bytes());
        out.extend_from_slice(&compressed);

        Ok(out)
    }

    fn decompress(&self, data: &[u8]) -> Result<Vec<u8>> {
        if data.len() < 2 {
            bail!("Input too short for zlib decompression");
        }
        let mut decoder = ZlibDecoder::new(&data[2..]);
        let mut decompressed = Vec::new();
        decoder.read_to_end(&mut decompressed)?;
        Ok(decompressed)
    }
}

// ============================================================================
// Cascading Pipeline
// ============================================================================
pub struct UFOPipeline {
    stages: Vec<Box<dyn CompressionStage>>,
}

pub type CascadeIntermediates = Vec<(String, usize)>;

impl UFOPipeline {
    pub fn new(stages: Vec<Box<dyn CompressionStage>>) -> Self {
        Self { stages }
    }

    pub fn compress(&self, data: &[u8]) -> Result<(Vec<u8>, CascadeIntermediates)> {
        let mut current = data.to_vec();
        let mut intermediates = vec![("Input".to_string(), data.len())];

        for (i, stage) in self.stages.iter().enumerate() {
            current = stage.compress(&current)?;
            intermediates.push((format!("Level {}", i + 1), current.len()));
        }

        Ok((current, intermediates))
    }

    pub fn decompress(&self, data: &[u8]) -> Result<Vec<u8>> {
        let mut current = data.to_vec();
        for stage in self.stages.iter().rev() {
            current = stage.decompress(&current)?;
        }
        Ok(current)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn get_full_pipeline() -> UFOPipeline {
        UFOPipeline::new(vec![
            Box::new(Level1Tokenizer),
            Box::new(Level2PrefixSuffix),
            Box::new(Level3Delta),
            Box::new(Level4Rle),
            Box::new(Level5FreqReorder),
            Box::new(Level6Zlib),
        ])
    }

    #[test]
    fn test_cascade_empty_input() -> Result<()> {
        let pipeline = get_full_pipeline();
        let input = b"";
        let (compressed, _) = pipeline.compress(input)?;
        let decompressed = pipeline.decompress(&compressed)?;
        assert_eq!(input.to_vec(), decompressed);
        Ok(())
    }

    #[test]
    fn test_cascade_lossless_text() -> Result<()> {
        let pipeline = get_full_pipeline();
        let input = "Hello,   World!\nThis is a\tlossless test.\n\n   Spaces galore!   ";
        let (compressed, _) = pipeline.compress(input.as_bytes())?;
        let decompressed = pipeline.decompress(&compressed)?;
        assert_eq!(input.as_bytes(), decompressed.as_slice());
        Ok(())
    }

    #[test]
    fn test_cascade_all_256_byte_values() -> Result<()> {
        let pipeline = get_full_pipeline();
        let mut input = Vec::with_capacity(256);
        for i in 0..=255 {
            input.push(i as u8);
        }
        let (compressed, _) = pipeline.compress(&input)?;
        let decompressed = pipeline.decompress(&compressed)?;
        assert_eq!(input, decompressed);
        Ok(())
    }

    #[test]
    fn test_cascade_large_dictionary() -> Result<()> {
        let pipeline = get_full_pipeline();
        // Generate > 256 unique words / chunks
        let mut text = String::new();
        for i in 0..300 {
            text.push_str(&format!("word{} ", i));
        }
        let input = text.trim_end();
        let (compressed, _) = pipeline.compress(input.as_bytes())?;
        let decompressed = pipeline.decompress(&compressed)?;
        assert_eq!(input.as_bytes(), decompressed.as_slice());
        Ok(())
    }

    #[test]
    fn test_cascade_malformed_streams() {
        let pipeline = get_full_pipeline();
        // Passing random garbage or truncated headers should fail gracefully
        assert!(pipeline.decompress(&[0]).is_err());
        assert!(pipeline.decompress(&[0, 1]).is_err());
        assert!(pipeline.decompress(&[0, 5, 255]).is_err());
    }
}

// ============================================================================
// Model Fallback & Cascading Circuit Breaker
// ============================================================================

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
pub enum ModelTier {
    LocalGpu,
    LocalCpu,
    P2pSwarm,
    RemoteMcp,
}

#[derive(Debug, Clone)]
pub struct ModelEndpoint {
    pub name: String,
    pub tier: ModelTier,
    pub enabled: bool,
    pub max_consecutive_failures: usize,
}

pub struct ModelFallbackChain {
    endpoints: Vec<ModelEndpoint>,
    failures: std::sync::Arc<std::sync::Mutex<HashMap<String, usize>>>,
}

impl ModelFallbackChain {
    pub fn new(endpoints: Vec<ModelEndpoint>) -> Self {
        Self {
            endpoints,
            failures: std::sync::Arc::new(std::sync::Mutex::new(HashMap::new())),
        }
    }

    pub fn default_cascade() -> Self {
        Self::new(vec![
            ModelEndpoint {
                name: "local-gpu-gguf".to_string(),
                tier: ModelTier::LocalGpu,
                enabled: true,
                max_consecutive_failures: 3,
            },
            ModelEndpoint {
                name: "local-cpu-quantized".to_string(),
                tier: ModelTier::LocalCpu,
                enabled: true,
                max_consecutive_failures: 3,
            },
            ModelEndpoint {
                name: "p2p-swarm-node".to_string(),
                tier: ModelTier::P2pSwarm,
                enabled: true,
                max_consecutive_failures: 3,
            },
            ModelEndpoint {
                name: "remote-mcp-server".to_string(),
                tier: ModelTier::RemoteMcp,
                enabled: true,
                max_consecutive_failures: 3,
            },
        ])
    }

    pub fn execute_with_fallback<F>(&self, mut generate_fn: F) -> Result<(String, ModelTier)>
    where
        F: FnMut(&ModelEndpoint) -> Result<String>,
    {
        let failures_map = self.failures.lock().unwrap();

        for endpoint in &self.endpoints {
            if !endpoint.enabled {
                continue;
            }

            let fail_count = failures_map.get(&endpoint.name).copied().unwrap_or(0);
            if fail_count >= endpoint.max_consecutive_failures {
                continue; // Circuit breaker open
            }

            drop(failures_map); // Release lock before running generation

            match generate_fn(endpoint) {
                Ok(output) => {
                    let mut lock = self.failures.lock().unwrap();
                    lock.insert(endpoint.name.clone(), 0); // Reset failure count on success
                    return Ok((output, endpoint.tier));
                }
                Err(_err) => {
                    let mut lock = self.failures.lock().unwrap();
                    let count = lock.entry(endpoint.name.clone()).or_insert(0);
                    *count += 1;
                    // Fallback to next tier
                }
            }

            return self.execute_with_fallback(generate_fn);
        }

        bail!("All model endpoints in fallback cascade failed or circuit-breaker open")
    }
}

#[cfg(test)]
mod fallback_tests {
    use super::*;

    #[test]
    fn test_model_fallback_cascade_switches_tiers() -> Result<()> {
        let chain = ModelFallbackChain::default_cascade();

        // Simulate local GPU failure, leading to local CPU success
        let (output, tier) = chain.execute_with_fallback(|ep| {
            if ep.tier == ModelTier::LocalGpu {
                bail!("GPU Out of Memory");
            } else {
                Ok(format!("Generated by {}", ep.name))
            }
        })?;

        assert_eq!(tier, ModelTier::LocalCpu);
        assert!(output.contains("local-cpu-quantized"));
        Ok(())
    }
}
