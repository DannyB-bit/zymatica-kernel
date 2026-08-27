use anyhow::{Context, Result, anyhow, bail};
use reed_solomon_erasure::galois_8::ReedSolomon;
use sha2::{Digest, Sha256};

pub const SYNC_MARKER: u8 = 0xBB;
pub const PACKET_SIZE: usize = 255;
pub const HEADER_SIZE: usize = 3;
pub const DATA_PER_PACKET: usize = PACKET_SIZE - HEADER_SIZE;
const OTA_KV_MAGIC: &[u8; 8] = b"ZKVOTA01";

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ChirpPacket {
    pub index: u8,
    pub total: u8,
    pub payload: [u8; DATA_PER_PACKET],
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct OtaKvSnapshot {
    pub sequence_id: u64,
    pub sha256: [u8; 32],
    pub bytes: Vec<u8>,
}

impl ChirpPacket {
    pub fn to_bytes(&self) -> [u8; PACKET_SIZE] {
        let mut out = [0_u8; PACKET_SIZE];
        out[0] = SYNC_MARKER;
        out[1] = self.index;
        out[2] = self.total;
        out[HEADER_SIZE..].copy_from_slice(&self.payload);
        out
    }

    pub fn from_bytes(bytes: &[u8]) -> Option<Self> {
        if bytes.len() != PACKET_SIZE || bytes[0] != SYNC_MARKER {
            return None;
        }
        let mut payload = [0_u8; DATA_PER_PACKET];
        payload.copy_from_slice(&bytes[HEADER_SIZE..]);
        Some(Self {
            index: bytes[1],
            total: bytes[2],
            payload,
        })
    }
}

pub fn pack_with_single_xor_fec(payload: &[u8], data_packets: usize) -> Vec<ChirpPacket> {
    assert!(data_packets > 0);
    assert!(data_packets < u8::MAX as usize);
    let total = data_packets + 1;
    assert!(total <= u8::MAX as usize);

    let mut packets = Vec::with_capacity(total);
    for idx in 0..data_packets {
        let start = idx * DATA_PER_PACKET;
        let end = (start + DATA_PER_PACKET).min(payload.len());
        let mut chunk = [0_u8; DATA_PER_PACKET];
        if start < payload.len() {
            chunk[..end - start].copy_from_slice(&payload[start..end]);
        }
        packets.push(ChirpPacket {
            index: idx as u8,
            total: total as u8,
            payload: chunk,
        });
    }

    let mut parity = [0_u8; DATA_PER_PACKET];
    for packet in &packets {
        for (dst, src) in parity.iter_mut().zip(packet.payload) {
            *dst ^= src;
        }
    }
    packets.push(ChirpPacket {
        index: data_packets as u8,
        total: total as u8,
        payload: parity,
    });
    packets
}

pub fn pack_with_reed_solomon_fec(
    payload: &[u8],
    data_packets: usize,
    parity_packets: usize,
) -> Result<Vec<ChirpPacket>> {
    validate_reed_solomon_layout(data_packets, parity_packets)?;
    let total = data_packets + parity_packets;
    let mut shards = vec![vec![0_u8; DATA_PER_PACKET]; total];
    for (idx, shard) in shards.iter_mut().take(data_packets).enumerate() {
        let start = idx * DATA_PER_PACKET;
        let end = (start + DATA_PER_PACKET).min(payload.len());
        if start < payload.len() {
            shard[..end - start].copy_from_slice(&payload[start..end]);
        }
    }

    let codec = ReedSolomon::new(data_packets, parity_packets)
        .map_err(|err| anyhow!("creating Reed-Solomon encoder: {err:?}"))?;
    codec
        .encode(&mut shards)
        .map_err(|err| anyhow!("encoding Reed-Solomon parity shards: {err:?}"))?;

    Ok(shards
        .into_iter()
        .enumerate()
        .map(|(idx, shard)| {
            let mut chunk = [0_u8; DATA_PER_PACKET];
            chunk.copy_from_slice(&shard);
            ChirpPacket {
                index: idx as u8,
                total: total as u8,
                payload: chunk,
            }
        })
        .collect())
}

pub fn recover_reed_solomon(
    received: &[ChirpPacket],
    data_packets: usize,
    parity_packets: usize,
) -> Result<Vec<ChirpPacket>> {
    validate_reed_solomon_layout(data_packets, parity_packets)?;
    let total = data_packets + parity_packets;
    if received.len() < data_packets {
        bail!(
            "not enough packets to recover: have {}, need at least {}",
            received.len(),
            data_packets
        );
    }

    let mut shards = vec![None; total];
    for packet in received {
        let idx = packet.index as usize;
        if packet.total as usize != total {
            bail!(
                "packet total mismatch: expected {}, got {}",
                total,
                packet.total
            );
        }
        if idx >= total {
            bail!("packet index {} out of range for total {}", idx, total);
        }
        if shards[idx].is_some() {
            bail!("duplicate packet index {}", idx);
        }
        shards[idx] = Some(packet.payload.to_vec());
    }

    let missing = shards.iter().filter(|shard| shard.is_none()).count();
    if missing > parity_packets {
        bail!(
            "too many erasures for Reed-Solomon recovery: missing {}, parity {}",
            missing,
            parity_packets
        );
    }

    let codec = ReedSolomon::new(data_packets, parity_packets)
        .map_err(|err| anyhow!("creating Reed-Solomon decoder: {err:?}"))?;
    codec
        .reconstruct(&mut shards)
        .map_err(|err| anyhow!("reconstructing Reed-Solomon shards: {err:?}"))?;

    let mut packets = Vec::with_capacity(total);
    for (idx, shard) in shards.into_iter().enumerate() {
        let shard = shard.context("Reed-Solomon reconstruction left missing shard")?;
        if shard.len() != DATA_PER_PACKET {
            bail!(
                "reconstructed shard {} has invalid size {}, expected {}",
                idx,
                shard.len(),
                DATA_PER_PACKET
            );
        }
        let mut payload = [0_u8; DATA_PER_PACKET];
        payload.copy_from_slice(&shard);
        packets.push(ChirpPacket {
            index: idx as u8,
            total: total as u8,
            payload,
        });
    }
    Ok(packets)
}

fn validate_reed_solomon_layout(data_packets: usize, parity_packets: usize) -> Result<()> {
    if data_packets == 0 {
        bail!("data_packets must be greater than zero");
    }
    if parity_packets == 0 {
        bail!("parity_packets must be greater than zero");
    }
    let total = data_packets
        .checked_add(parity_packets)
        .context("packet count overflow")?;
    if total > u8::MAX as usize {
        bail!(
            "total packet count {} exceeds u8 packet header capacity",
            total
        );
    }
    Ok(())
}

pub fn recover_single_missing(received: &[ChirpPacket]) -> Option<Vec<ChirpPacket>> {
    if received.is_empty() {
        return None;
    }
    let total = received[0].total as usize;
    if received.len() != total - 1 {
        return None;
    }
    if received.iter().any(|packet| packet.total as usize != total) {
        return None;
    }
    let mut seen = vec![false; total];
    for packet in received {
        let idx = packet.index as usize;
        if idx >= total || seen[idx] {
            return None;
        }
        seen[idx] = true;
    }
    let missing = seen.iter().position(|v| !*v)?;
    let mut recovered_payload = [0_u8; DATA_PER_PACKET];
    for packet in received {
        for (dst, src) in recovered_payload.iter_mut().zip(packet.payload) {
            *dst ^= src;
        }
    }
    let mut all = received.to_vec();
    all.push(ChirpPacket {
        index: missing as u8,
        total: total as u8,
        payload: recovered_payload,
    });
    all.sort_by_key(|packet| packet.index);
    Some(all)
}

pub fn reassemble_data_packets(packets: &[ChirpPacket], original_len: usize) -> Vec<u8> {
    let mut sorted = packets.to_vec();
    sorted.sort_by_key(|packet| packet.index);
    let parity_index = sorted.first().map(|p| p.total as usize - 1).unwrap_or(0);
    let mut out = Vec::new();
    for packet in sorted {
        if packet.index as usize == parity_index {
            continue;
        }
        out.extend_from_slice(&packet.payload);
    }
    out.truncate(original_len);
    out
}

pub fn encode_ota_kv_snapshot(sequence_id: u64, snapshot: &[u8]) -> Vec<u8> {
    let digest = Sha256::digest(snapshot);
    let mut out = Vec::with_capacity(8 + 8 + 8 + 32 + snapshot.len());
    out.extend_from_slice(OTA_KV_MAGIC);
    out.extend_from_slice(&sequence_id.to_le_bytes());
    out.extend_from_slice(&(snapshot.len() as u64).to_le_bytes());
    out.extend_from_slice(&digest);
    out.extend_from_slice(snapshot);
    out
}

pub fn decode_ota_kv_snapshot(payload: &[u8]) -> Option<OtaKvSnapshot> {
    if payload.len() < 56 || &payload[..8] != OTA_KV_MAGIC {
        return None;
    }
    let sequence_id = u64::from_le_bytes(payload[8..16].try_into().ok()?);
    let len = u64::from_le_bytes(payload[16..24].try_into().ok()?) as usize;
    let sha256: [u8; 32] = payload[24..56].try_into().ok()?;
    let end = 56_usize.checked_add(len)?;
    if payload.len() < end {
        return None;
    }
    if payload[end..].iter().any(|byte| *byte != 0) {
        return None;
    }
    let bytes = payload[56..end].to_vec();
    if Sha256::digest(&bytes).as_slice() != sha256 {
        return None;
    }
    Some(OtaKvSnapshot {
        sequence_id,
        sha256,
        bytes,
    })
}

pub fn pack_ota_kv_snapshot(sequence_id: u64, snapshot: &[u8]) -> Vec<ChirpPacket> {
    let payload = encode_ota_kv_snapshot(sequence_id, snapshot);
    let data_packets = payload.len().div_ceil(DATA_PER_PACKET);
    pack_with_single_xor_fec(&payload, data_packets)
}

pub fn pack_ota_kv_snapshot_reed_solomon(
    sequence_id: u64,
    snapshot: &[u8],
    parity_packets: usize,
) -> Result<Vec<ChirpPacket>> {
    let payload = encode_ota_kv_snapshot(sequence_id, snapshot);
    let data_packets = payload.len().div_ceil(DATA_PER_PACKET);
    pack_with_reed_solomon_fec(&payload, data_packets, parity_packets)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn xor_fec_recovers_one_missing_packet() {
        let payload = b"ip zymatica.space | zk lorawan field packet ".repeat(31);
        let data_packets = payload.len().div_ceil(DATA_PER_PACKET);
        let packets = pack_with_single_xor_fec(&payload, data_packets);
        let received: Vec<_> = packets
            .iter()
            .enumerate()
            .filter_map(|(idx, packet)| (idx != 2).then_some(packet.clone()))
            .collect();
        let healed = recover_single_missing(&received).unwrap();
        let reassembled = reassemble_data_packets(&healed, payload.len());
        assert_eq!(reassembled, payload);
    }

    #[test]
    fn reed_solomon_recovers_multiple_missing_packets() {
        let payload = b"burst-loss telemetry payload ".repeat(77);
        let data_packets = payload.len().div_ceil(DATA_PER_PACKET);
        let packets = pack_with_reed_solomon_fec(&payload, data_packets, 3).unwrap();
        let received: Vec<_> = packets
            .iter()
            .filter(|packet| !matches!(packet.index, 1 | 3 | 5))
            .cloned()
            .collect();
        let healed = recover_reed_solomon(&received, data_packets, 3).unwrap();
        let reassembled = reassemble_data_packets(&healed, payload.len());
        assert_eq!(reassembled, payload);
    }

    #[test]
    fn reed_solomon_refuses_unrecoverable_burst_loss() {
        let payload = b"loss budget exceeded ".repeat(64);
        let data_packets = payload.len().div_ceil(DATA_PER_PACKET);
        let packets = pack_with_reed_solomon_fec(&payload, data_packets, 2).unwrap();
        let received: Vec<_> = packets
            .iter()
            .filter(|packet| !matches!(packet.index, 0..=2))
            .cloned()
            .collect();
        assert!(recover_reed_solomon(&received, data_packets, 2).is_err());
    }

    #[test]
    fn packet_serialization_round_trip() {
        let packets = pack_with_single_xor_fec(b"hello", 1);
        let bytes = packets[0].to_bytes();
        assert_eq!(ChirpPacket::from_bytes(&bytes), Some(packets[0].clone()));
    }

    #[test]
    fn ota_kv_snapshot_survives_single_packet_loss() {
        let snapshot = b"kv snapshot bytes from a spilled sequence".repeat(20);
        let packets = pack_ota_kv_snapshot(99, &snapshot);
        let original_len = encode_ota_kv_snapshot(99, &snapshot).len();
        let received: Vec<_> = packets
            .iter()
            .filter(|packet| packet.index != 1)
            .cloned()
            .collect();
        let healed = recover_single_missing(&received).unwrap();
        let payload = reassemble_data_packets(&healed, original_len);
        let decoded = decode_ota_kv_snapshot(&payload).unwrap();
        assert_eq!(decoded.sequence_id, 99);
        assert_eq!(decoded.bytes, snapshot);
    }

    #[test]
    fn ota_kv_snapshot_survives_reed_solomon_burst_loss() {
        let snapshot = b"rs kv snapshot bytes from a spilled sequence".repeat(30);
        let packets = pack_ota_kv_snapshot_reed_solomon(101, &snapshot, 4).unwrap();
        let original_len = encode_ota_kv_snapshot(101, &snapshot).len();
        let data_packets = original_len.div_ceil(DATA_PER_PACKET);
        let received: Vec<_> = packets
            .iter()
            .filter(|packet| !matches!(packet.index, 2 | 4 | 6 | 8))
            .cloned()
            .collect();
        let healed = recover_reed_solomon(&received, data_packets, 4).unwrap();
        let payload = reassemble_data_packets(&healed, original_len);
        let decoded = decode_ota_kv_snapshot(&payload).unwrap();
        assert_eq!(decoded.sequence_id, 101);
        assert_eq!(decoded.bytes, snapshot);
    }

    #[test]
    fn ota_kv_snapshot_decoder_accepts_zero_packet_padding() {
        let snapshot = b"short kv snapshot";
        let mut payload = encode_ota_kv_snapshot(7, snapshot);
        payload.resize(DATA_PER_PACKET, 0);
        let decoded = decode_ota_kv_snapshot(&payload).unwrap();
        assert_eq!(decoded.sequence_id, 7);
        assert_eq!(decoded.bytes, snapshot);
    }
}
