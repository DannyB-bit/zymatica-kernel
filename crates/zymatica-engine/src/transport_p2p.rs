//! Peer-to-Peer speculative draft clustering and erasure beam-search transport.

use crate::paged_kv::KvCachePacket;
use anyhow::{Context, Result, anyhow, bail};
use reed_solomon_erasure::galois_8::ReedSolomon;
use sha2::{Digest, Sha256};
use std::collections::HashMap;
use std::net::SocketAddr;

const BEAM_FRAME_HEADER_BYTES: usize = 8;

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct P2pClusterNode {
    pub addr: SocketAddr,
    pub capacity: usize,
    pub priority: u8,
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct P2pKvSwapPeer {
    pub peer_id: String,
    pub capacity_bytes: usize,
    pub resident_bytes: usize,
    pub priority: u8,
}

impl P2pKvSwapPeer {
    pub fn free_bytes(&self) -> usize {
        self.capacity_bytes.saturating_sub(self.resident_bytes)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct P2pKvSwapManifest {
    pub sequence_id: u64,
    pub token_len: usize,
    pub page_count: usize,
    pub page_size: usize,
    pub compact: bool,
    pub byte_len: usize,
    pub sha256: String,
    pub peer_id: String,
}

#[derive(Debug, Default)]
pub struct P2pKvSwapStore {
    peers: HashMap<String, P2pKvSwapPeer>,
    packets: HashMap<(String, u64), KvCachePacket>,
}

impl P2pKvSwapStore {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn register_peer(
        &mut self,
        peer_id: impl Into<String>,
        capacity_bytes: usize,
        priority: u8,
    ) -> Result<()> {
        if capacity_bytes == 0 {
            bail!("p2p kv swap peer capacity must be greater than zero");
        }
        let peer_id = peer_id.into();
        let resident_bytes = self
            .packets
            .iter()
            .filter(|((packet_peer_id, _), _)| *packet_peer_id == peer_id)
            .map(|(_, packet)| packet.bytes.len())
            .sum();
        if resident_bytes > capacity_bytes {
            bail!(
                "peer {peer_id} already holds {resident_bytes} bytes, exceeding new capacity {capacity_bytes}"
            );
        }
        self.peers.insert(
            peer_id.clone(),
            P2pKvSwapPeer {
                peer_id,
                capacity_bytes,
                resident_bytes,
                priority,
            },
        );
        Ok(())
    }

    pub fn peer(&self, peer_id: &str) -> Option<&P2pKvSwapPeer> {
        self.peers.get(peer_id)
    }

    pub fn peer_count(&self) -> usize {
        self.peers.len()
    }

    pub fn resident_packet_count(&self) -> usize {
        self.packets.len()
    }

    pub fn resident_bytes(&self) -> usize {
        self.peers.values().map(|peer| peer.resident_bytes).sum()
    }

    pub fn stream_out_packet(&mut self, packet: KvCachePacket) -> Result<P2pKvSwapManifest> {
        verify_packet_hash(&packet)?;
        let byte_len = packet.bytes.len();
        if byte_len == 0 {
            bail!("cannot stream an empty kv cache packet");
        }
        let peer_id = self
            .peers
            .values()
            .filter(|peer| peer.free_bytes() >= byte_len)
            .max_by_key(|peer| (peer.priority, peer.free_bytes()))
            .map(|peer| peer.peer_id.clone())
            .with_context(|| {
                format!("no p2p peer has {byte_len} bytes of free KV swap capacity")
            })?;

        let key = (peer_id.clone(), packet.sequence_id);
        let old_len = self
            .packets
            .get(&key)
            .map(|old| old.bytes.len())
            .unwrap_or(0);
        let peer = self
            .peers
            .get_mut(&peer_id)
            .context("selected p2p peer disappeared before stream-out")?;
        let adjusted_resident = peer.resident_bytes.saturating_sub(old_len);
        if adjusted_resident + byte_len > peer.capacity_bytes {
            bail!(
                "peer {peer_id} capacity changed during stream-out: need {} free bytes, have {}",
                byte_len,
                peer.capacity_bytes.saturating_sub(adjusted_resident)
            );
        }
        peer.resident_bytes = adjusted_resident + byte_len;
        self.packets.insert(key, packet.clone());

        Ok(P2pKvSwapManifest {
            sequence_id: packet.sequence_id,
            token_len: packet.token_len,
            page_count: packet.page_count,
            page_size: packet.page_size,
            compact: packet.compact,
            byte_len,
            sha256: packet.sha256,
            peer_id,
        })
    }

    pub fn stream_in_packet(&self, manifest: &P2pKvSwapManifest) -> Result<KvCachePacket> {
        let packet = self
            .packets
            .get(&(manifest.peer_id.clone(), manifest.sequence_id))
            .with_context(|| {
                format!(
                    "p2p peer {} does not hold sequence {}",
                    manifest.peer_id, manifest.sequence_id
                )
            })?;
        if packet.token_len != manifest.token_len
            || packet.page_count != manifest.page_count
            || packet.page_size != manifest.page_size
            || packet.compact != manifest.compact
            || packet.bytes.len() != manifest.byte_len
            || packet.sha256 != manifest.sha256
        {
            bail!("p2p kv swap manifest does not match resident packet metadata");
        }
        verify_packet_hash(packet)?;
        Ok(packet.clone())
    }

    pub fn release(&mut self, manifest: &P2pKvSwapManifest) -> Result<()> {
        let key = (manifest.peer_id.clone(), manifest.sequence_id);
        let packet = self.packets.remove(&key).with_context(|| {
            format!(
                "sequence {} is not resident on p2p peer {}",
                manifest.sequence_id, manifest.peer_id
            )
        })?;
        let peer = self
            .peers
            .get_mut(&manifest.peer_id)
            .context("p2p peer disappeared before release")?;
        peer.resident_bytes = peer.resident_bytes.saturating_sub(packet.bytes.len());
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct SpeculativePathBranch {
    pub branch_id: u32,
    pub parent_branch_id: u32,
    pub tokens: Vec<usize>,
    pub confidence: f32,
}

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct SpeculativeBeamPayload {
    pub step_index: u64,
    pub base_tokens: Vec<usize>,
    pub branches: Vec<SpeculativePathBranch>,
}

#[derive(Debug)]
pub struct SpeculativeBeamErasureRing {
    pub payload: SpeculativeBeamPayload,
    pub data_shards: usize,
    pub parity_shards: usize,
}

impl SpeculativeBeamErasureRing {
    pub fn new(payload: SpeculativeBeamPayload, data_shards: usize, parity_shards: usize) -> Self {
        Self {
            payload,
            data_shards,
            parity_shards,
        }
    }

    pub fn encode_to_packets(&self) -> Result<Vec<Vec<u8>>> {
        validate_layout(self.data_shards, self.parity_shards)?;
        let serialized = serde_json::to_vec(&self.payload)?;
        let total_shards = self.data_shards + self.parity_shards;

        let mut padded = Vec::with_capacity(BEAM_FRAME_HEADER_BYTES + serialized.len());
        padded.extend_from_slice(&(serialized.len() as u64).to_le_bytes());
        padded.extend_from_slice(&serialized);

        let shard_size = padded
            .len()
            .div_ceil(self.data_shards)
            .max(BEAM_FRAME_HEADER_BYTES);
        padded.resize(shard_size * self.data_shards, 0);

        let mut shards = Vec::with_capacity(total_shards);
        for i in 0..self.data_shards {
            let start = i * shard_size;
            let end = start + shard_size;
            shards.push(padded[start..end].to_vec());
        }
        for _ in 0..self.parity_shards {
            shards.push(vec![0; shard_size]);
        }

        let codec = ReedSolomon::new(self.data_shards, self.parity_shards)
            .map_err(|err| anyhow!("creating speculative beam Reed-Solomon encoder: {err:?}"))?;
        let mut shard_refs: Vec<_> = shards.iter_mut().map(Vec::as_mut_slice).collect();
        codec
            .encode(&mut shard_refs)
            .map_err(|err| anyhow!("encoding speculative beam shards: {err:?}"))?;

        Ok(shards)
    }

    pub fn decode_from_packets(
        shards: &[Vec<u8>],
        data_shards: usize,
    ) -> Result<SpeculativeBeamPayload> {
        if shards.len() < data_shards {
            bail!(
                "not enough speculative beam shards: have {}, need at least {}",
                shards.len(),
                data_shards
            );
        }
        let shard_size = shards
            .first()
            .map(Vec::len)
            .context("speculative beam shard list is empty")?;
        if shards
            .iter()
            .take(data_shards)
            .any(|shard| shard.len() != shard_size)
        {
            bail!("speculative beam shards have inconsistent sizes");
        }

        let mut combined = Vec::new();
        for shard in shards.iter().take(data_shards) {
            combined.extend_from_slice(shard);
        }
        decode_beam_frame(&combined)
    }

    pub fn recover_from_packets(
        shards: &[Option<Vec<u8>>],
        data_shards: usize,
        parity_shards: usize,
    ) -> Result<SpeculativeBeamPayload> {
        validate_layout(data_shards, parity_shards)?;
        let total = data_shards + parity_shards;
        if shards.len() != total {
            bail!(
                "speculative beam shard count mismatch: expected {}, got {}",
                total,
                shards.len()
            );
        }
        let present = shards.iter().filter(|shard| shard.is_some()).count();
        if present < data_shards {
            bail!(
                "too many speculative beam erasures: have {}, need {}",
                present,
                data_shards
            );
        }

        let mut shards = shards.to_vec();
        let codec = ReedSolomon::new(data_shards, parity_shards)
            .map_err(|err| anyhow!("creating speculative beam Reed-Solomon decoder: {err:?}"))?;
        codec
            .reconstruct(&mut shards)
            .map_err(|err| anyhow!("reconstructing speculative beam shards: {err:?}"))?;

        let mut combined = Vec::new();
        for shard in shards.into_iter().take(data_shards) {
            combined.extend_from_slice(
                &shard.context("speculative beam reconstruction left a data shard missing")?,
            );
        }
        decode_beam_frame(&combined)
    }
}

fn validate_layout(data_shards: usize, parity_shards: usize) -> Result<()> {
    if data_shards == 0 {
        bail!("data_shards must be greater than zero");
    }
    if parity_shards == 0 {
        bail!("parity_shards must be greater than zero");
    }
    Ok(())
}

fn decode_beam_frame(frame: &[u8]) -> Result<SpeculativeBeamPayload> {
    if frame.len() < BEAM_FRAME_HEADER_BYTES {
        bail!("speculative beam frame is shorter than the length header");
    }
    let len = u64::from_le_bytes(frame[..BEAM_FRAME_HEADER_BYTES].try_into().unwrap()) as usize;
    let end = BEAM_FRAME_HEADER_BYTES
        .checked_add(len)
        .context("speculative beam frame length overflow")?;
    if end > frame.len() {
        bail!(
            "speculative beam frame length {} exceeds available {}",
            len,
            frame.len() - BEAM_FRAME_HEADER_BYTES
        );
    }
    Ok(serde_json::from_slice(
        &frame[BEAM_FRAME_HEADER_BYTES..end],
    )?)
}

fn verify_packet_hash(packet: &KvCachePacket) -> Result<()> {
    let sha256 = hex_digest(Sha256::digest(&packet.bytes).as_slice());
    if sha256 != packet.sha256 {
        bail!(
            "kv packet sha256 mismatch before p2p swap: expected {} got {sha256}",
            packet.sha256
        );
    }
    Ok(())
}

fn hex_digest(bytes: &[u8]) -> String {
    let mut out = String::with_capacity(bytes.len() * 2);
    for byte in bytes {
        out.push_str(&format!("{byte:02x}"));
    }
    out
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct MultiGpuShardManifest {
    pub sequence_id: u64,
    pub page_count: usize,
    pub page_size: usize,
    pub compact: bool,
    pub total_shards: usize,
    pub data_shards: usize,
    pub parity_shards: usize,
    pub payload_len: usize,
    pub sha256: String,
    pub gpu_device_ids: Vec<usize>,
}

#[derive(Debug)]
pub struct P2pMultiGpuRingSharder {
    data_shards: usize,
    parity_shards: usize,
    gpu_count: usize,
}

impl P2pMultiGpuRingSharder {
    pub fn new(data_shards: usize, parity_shards: usize, gpu_count: usize) -> Result<Self> {
        if data_shards == 0 || parity_shards == 0 {
            bail!("data and parity shard counts must be greater than zero");
        }
        if gpu_count == 0 {
            bail!("gpu_count must be at least 1");
        }
        Ok(Self {
            data_shards,
            parity_shards,
            gpu_count,
        })
    }

    pub fn shard_kv_packet(
        &self,
        packet: &KvCachePacket,
    ) -> Result<(MultiGpuShardManifest, Vec<Vec<u8>>)> {
        let total_shards = self.data_shards + self.parity_shards;
        let mut padded = Vec::with_capacity(8 + packet.bytes.len());
        padded.extend_from_slice(&(packet.bytes.len() as u64).to_le_bytes());
        padded.extend_from_slice(&packet.bytes);

        let shard_size = padded.len().div_ceil(self.data_shards).max(8);
        padded.resize(shard_size * self.data_shards, 0);

        let mut shards = Vec::with_capacity(total_shards);
        for i in 0..self.data_shards {
            let start = i * shard_size;
            shards.push(padded[start..start + shard_size].to_vec());
        }
        for _ in 0..self.parity_shards {
            shards.push(vec![0; shard_size]);
        }

        let codec = ReedSolomon::new(self.data_shards, self.parity_shards)
            .map_err(|err| anyhow!("creating Reed-Solomon encoder: {err:?}"))?;
        let mut shard_refs: Vec<_> = shards.iter_mut().map(Vec::as_mut_slice).collect();
        codec
            .encode(&mut shard_refs)
            .map_err(|err| anyhow!("encoding multi-GPU shards: {err:?}"))?;

        let gpu_device_ids = (0..total_shards).map(|i| i % self.gpu_count).collect();
        let manifest = MultiGpuShardManifest {
            sequence_id: packet.sequence_id,
            page_count: packet.page_count,
            page_size: packet.page_size,
            compact: packet.compact,
            total_shards,
            data_shards: self.data_shards,
            parity_shards: self.parity_shards,
            payload_len: packet.bytes.len(),
            sha256: packet.sha256.clone(),
            gpu_device_ids,
        };

        Ok((manifest, shards))
    }

    pub fn reconstruct_kv_packet(
        &self,
        manifest: &MultiGpuShardManifest,
        shards: &[Option<Vec<u8>>],
    ) -> Result<KvCachePacket> {
        let total_shards = manifest.data_shards + manifest.parity_shards;
        if shards.len() != total_shards {
            bail!("invalid shard slice length");
        }

        let mut work_shards: Vec<Option<Vec<u8>>> = shards.to_vec();
        let codec = ReedSolomon::new(manifest.data_shards, manifest.parity_shards)
            .map_err(|err| anyhow!("creating Reed-Solomon decoder: {err:?}"))?;
        codec
            .reconstruct(&mut work_shards)
            .map_err(|err| anyhow!("reconstructing multi-GPU shards: {err:?}"))?;

        let shard_size = work_shards
            .first()
            .and_then(Option::as_ref)
            .map(Vec::len)
            .context("no reconstructed shards")?;

        let mut reassembled = Vec::with_capacity(manifest.data_shards * shard_size);
        for shard in work_shards.iter().take(manifest.data_shards) {
            reassembled.extend_from_slice(shard.as_ref().context("missing data shard")?);
        }

        if reassembled.len() < 8 {
            bail!("reassembled payload too short");
        }
        let payload_len = u64::from_le_bytes(reassembled[..8].try_into().unwrap()) as usize;
        if payload_len != manifest.payload_len || reassembled.len() < 8 + payload_len {
            bail!("invalid reassembled payload length");
        }
        let bytes = reassembled[8..8 + payload_len].to_vec();

        let mut hasher = Sha256::new();
        hasher.update(&bytes);
        let sha256 = format!("{:x}", hasher.finalize());
        if sha256 != manifest.sha256 {
            bail!("reconstructed multi-GPU KV packet sha256 mismatch");
        }

        Ok(KvCachePacket {
            sequence_id: manifest.sequence_id,
            token_len: 0,
            page_count: manifest.page_count,
            page_size: manifest.page_size,
            compact: manifest.compact,
            bytes,
            sha256,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::paged_kv::PagedKvCache;

    fn payload() -> SpeculativeBeamPayload {
        SpeculativeBeamPayload {
            step_index: 42,
            base_tokens: vec![2, 236761, 108],
            branches: vec![
                SpeculativePathBranch {
                    branch_id: 1,
                    parent_branch_id: 0,
                    tokens: vec![1018, 8291],
                    confidence: 0.91,
                },
                SpeculativePathBranch {
                    branch_id: 2,
                    parent_branch_id: 0,
                    tokens: vec![236829, 808],
                    confidence: 0.62,
                },
            ],
        }
    }

    #[test]
    fn multi_gpu_p2p_ring_sharding_round_trip() {
        let mut source = PagedKvCache::new(2, 2, 3, 4);
        for pos in 0..5 {
            source.allocate_token(42);
            source.set_kv(
                42,
                pos,
                1,
                1,
                &[pos as f32, pos as f32 + 1.0, pos as f32 + 2.0],
                &[pos as f32 + 10.0, pos as f32 + 20.0, pos as f32 + 30.0],
            );
        }
        let packet = source.export_sequence_compact_packet(42).unwrap();
        let sharder = P2pMultiGpuRingSharder::new(3, 2, 4).unwrap();
        let (manifest, packets) = sharder.shard_kv_packet(&packet).unwrap();
        assert_eq!(manifest.gpu_device_ids, vec![0, 1, 2, 3, 0]);

        let mut received: Vec<_> = packets.into_iter().map(Some).collect();
        received[1] = None; // Simulate single GPU failure
        let restored_packet = sharder.reconstruct_kv_packet(&manifest, &received).unwrap();
        assert_eq!(restored_packet.sha256, packet.sha256);
    }

    #[test]
    fn p2p_beam_packets_round_trip_without_padding_leak() {
        let ring = SpeculativeBeamErasureRing::new(payload(), 3, 2);
        let shards = ring.encode_to_packets().unwrap();
        let decoded = SpeculativeBeamErasureRing::decode_from_packets(&shards, 3).unwrap();
        assert_eq!(decoded, ring.payload);
    }

    #[test]
    fn p2p_beam_reed_solomon_recovers_lost_shards() {
        let ring = SpeculativeBeamErasureRing::new(payload(), 4, 3);
        let shards = ring.encode_to_packets().unwrap();
        let mut received: Vec<_> = shards.into_iter().map(Some).collect();
        received[1] = None;
        received[5] = None;
        let decoded = SpeculativeBeamErasureRing::recover_from_packets(&received, 4, 3).unwrap();
        assert_eq!(decoded, ring.payload);
    }

    #[test]
    fn p2p_kv_swap_streams_packet_to_peer_ram_and_restores() {
        let mut source = PagedKvCache::new(2, 2, 3, 4);
        for pos in 0..7 {
            source.allocate_token(9001);
            source.set_kv(
                9001,
                pos,
                1,
                1,
                &[pos as f32, pos as f32 + 0.5, pos as f32 + 1.0],
                &[pos as f32 + 10.0, pos as f32 + 20.0, pos as f32 + 30.0],
            );
        }

        let packet = source.export_sequence_compact_packet(9001).unwrap();
        let packet_sha = packet.sha256.clone();
        let mut store = P2pKvSwapStore::new();
        store
            .register_peer("laptop-ram", packet.bytes.len() + 128, 10)
            .unwrap();
        let manifest = store.stream_out_packet(packet).unwrap();
        source.free_sequence(9001);
        assert_eq!(source.resident_pages(), 0);
        assert_eq!(store.resident_packet_count(), 1);
        assert_eq!(manifest.sha256, packet_sha);

        let restored_packet = store.stream_in_packet(&manifest).unwrap();
        let mut restored = PagedKvCache::new(2, 2, 3, 4);
        restored.import_sequence_packet(&restored_packet).unwrap();
        assert_eq!(restored.key(9001, 6, 1, 1), &[6.0, 6.5, 7.0]);
        assert_eq!(restored.value(9001, 6, 1, 1), &[16.0, 26.0, 36.0]);
        store.release(&manifest).unwrap();
        assert_eq!(store.resident_bytes(), 0);
    }
}
