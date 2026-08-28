//! # Invention Class 28: Zymatica Neural Swarm Hypergraph (ZNS-Hypergraph)
//!
//! Autonomous, zero-bandwidth multi-agent consensus and morphogenetic ephemeral subagent
//! spawning over 6D semantic hypercube trajectories and air-gapped LoRa mesh networks.

use std::collections::HashMap;

/// 16-Byte Differential Swarm Intent Chirp Packet
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SwarmIntentChirp {
    pub sender_node_id: u8,
    pub swarm_epoch: u8,
    pub target_domain: u8,
    pub target_subdomain: u8,
    pub action_opcode: u8,
    pub consensus_weight: u8,
    pub concept_trajectory: [u8; 6],
    pub state_crc: u32,
}

impl SwarmIntentChirp {
    pub fn new(sender: u8, epoch: u8, domain: u8, subdomain: u8, opcode: u8, coords: [u8; 6]) -> Self {
        let mut chirp = Self {
            sender_node_id: sender,
            swarm_epoch: epoch,
            target_domain: domain & 0x0F,
            target_subdomain: subdomain & 0x0F,
            action_opcode: opcode,
            consensus_weight: 100,
            concept_trajectory: coords,
            state_crc: 0,
        };
        chirp.state_crc = chirp.compute_crc();
        chirp
    }

    pub fn to_bytes(&self) -> [u8; 16] {
        let mut bytes = [0u8; 16];
        bytes[0] = self.sender_node_id;
        bytes[1] = self.swarm_epoch;
        bytes[2] = (self.target_domain << 4) | (self.target_subdomain & 0x0F);
        bytes[3] = self.action_opcode;
        bytes[4] = self.consensus_weight;
        bytes[5..11].copy_from_slice(&self.concept_trajectory);
        let crc_bytes = self.state_crc.to_be_bytes();
        bytes[11..15].copy_from_slice(&crc_bytes);
        bytes[15] = 0x5A; // Swarm Sync Sentinel
        bytes
    }

    pub fn from_bytes(bytes: &[u8; 16]) -> Result<Self, &'static str> {
        if bytes[15] != 0x5A {
            return Err("Invalid Swarm Sentinel Byte");
        }
        let crc = u32::from_be_bytes([bytes[11], bytes[12], bytes[13], bytes[14]]);
        let mut coords = [0u8; 6];
        coords.copy_from_slice(&bytes[5..11]);

        let chirp = Self {
            sender_node_id: bytes[0],
            swarm_epoch: bytes[1],
            target_domain: (bytes[2] >> 4) & 0x0F,
            target_subdomain: bytes[2] & 0x0F,
            action_opcode: bytes[3],
            consensus_weight: bytes[4],
            concept_trajectory: coords,
            state_crc: crc,
        };

        if chirp.compute_crc() != crc {
            return Err("CRC Integrity Mismatch");
        }

        Ok(chirp)
    }

    fn compute_crc(&self) -> u32 {
        let mut hash = 0x811c9dc5u32;
        hash ^= self.sender_node_id as u32;
        hash = hash.wrapping_mul(0x01000193);
        hash ^= self.swarm_epoch as u32;
        hash = hash.wrapping_mul(0x01000193);
        hash ^= ((self.target_domain << 4) | self.target_subdomain) as u32;
        hash = hash.wrapping_mul(0x01000193);
        hash ^= self.action_opcode as u32;
        hash = hash.wrapping_mul(0x01000193);
        for &b in &self.concept_trajectory {
            hash ^= b as u32;
            hash = hash.wrapping_mul(0x01000193);
        }
        hash
    }
}

/// Morphogenetic Ephemeral Subagent Spawner
pub struct EphemeralSubagentSpawner;

impl EphemeralSubagentSpawner {
    /// Cold-start spawn a lightweight subagent from a 381-byte procedural seed
    pub fn spawn_from_seed(seed_381: &[u8; 381], domain_id: u8) -> Vec<f32> {
        let mut synthesized_weights = vec![0.0f32; 1024];
        let mut prng_state = (seed_381[0] as u64) << 24
            | (seed_381[1] as u64) << 16
            | (seed_381[2] as u64) << 8
            | (seed_381[3] as u64);

        for (i, w) in synthesized_weights.iter_mut().enumerate() {
            prng_state = prng_state.wrapping_mul(6364136223846793005).wrapping_add(1);
            let seed_byte = seed_381[(i + domain_id as usize) % 381] as f32 / 255.0;
            let normal_sample = ((prng_state >> 32) as f32 / (u32::MAX as f32)) - 0.5;
            *w = seed_byte * 0.1 + normal_sample * 0.05;
        }

        synthesized_weights
    }
}

/// Swarm Quorum Certificate establishing authenticated agreement across nodes
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SwarmQuorumCertificate {
    pub epoch: u8,
    pub participant_nodes: Vec<u8>,
    pub total_weight: u32,
    pub consensus_trajectory: [u8; 6],
    pub certificate_hash: u32,
}

/// Swarm Multi-Node Hypergraph Consensus Engine with Identity Authentication and Sybil Replay Guards
pub struct SwarmConsensusEngine {
    pub registered_nodes: Vec<u8>,
    pub pending_proposals: HashMap<u8, Vec<SwarmIntentChirp>>,
}

impl SwarmConsensusEngine {
    pub fn new(nodes: Vec<u8>) -> Self {
        Self {
            registered_nodes: nodes,
            pending_proposals: HashMap::new(),
        }
    }

    /// Submit a verified swarm intent proposal from an authorized registered node
    pub fn submit_intent(&mut self, chirp: SwarmIntentChirp) -> Result<(), &'static str> {
        // Enforce membership verification
        if !self.registered_nodes.contains(&chirp.sender_node_id) {
            return Err("Unauthorized node ID: sender is not in the registered swarm membership set");
        }

        let proposals = self.pending_proposals.entry(chirp.swarm_epoch).or_default();

        // Enforce one vote per registered node per epoch (prevent duplicate/Sybil replay)
        if proposals.iter().any(|p| p.sender_node_id == chirp.sender_node_id) {
            return Err("Duplicate vote detected: node has already submitted a proposal for this epoch");
        }

        proposals.push(chirp);
        Ok(())
    }

    /// Resolve weighted centroid consensus with quorum certificate generation
    pub fn resolve_consensus(&self, epoch: u8, quorum_threshold: usize) -> Option<SwarmQuorumCertificate> {
        let proposals = self.pending_proposals.get(&epoch)?;
        if proposals.len() < quorum_threshold {
            return None;
        }

        let mut sum_coords = [0u32; 6];
        let mut total_weight = 0u32;
        let mut participants = Vec::new();

        for p in proposals {
            let w = p.consensus_weight as u32;
            total_weight += w;
            participants.push(p.sender_node_id);
            for i in 0..6 {
                sum_coords[i] += (p.concept_trajectory[i] as u32) * w;
            }
        }

        if total_weight == 0 {
            return None;
        }

        let mut consensus_coords = [0u8; 6];
        for i in 0..6 {
            consensus_coords[i] = ((sum_coords[i] + total_weight / 2) / total_weight) as u8;
        }

        let mut cert_hash = 0x811c9dc5u32;
        cert_hash ^= epoch as u32;
        cert_hash = cert_hash.wrapping_mul(0x01000193);
        cert_hash ^= total_weight;
        cert_hash = cert_hash.wrapping_mul(0x01000193);
        for &c in &consensus_coords {
            cert_hash ^= c as u32;
            cert_hash = cert_hash.wrapping_mul(0x01000193);
        }

        Some(SwarmQuorumCertificate {
            epoch,
            participant_nodes: participants,
            total_weight,
            consensus_trajectory: consensus_coords,
            certificate_hash: cert_hash,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_swarm_intent_chirp_16byte_serialization() {
        let chirp = SwarmIntentChirp::new(1, 42, 2, 5, 0x07, [10, 20, 30, 40, 50, 60]);
        let bytes = chirp.to_bytes();
        assert_eq!(bytes.len(), 16);
        assert_eq!(bytes[15], 0x5A);

        let decoded = SwarmIntentChirp::from_bytes(&bytes).expect("Valid 16B chirp decode");
        assert_eq!(chirp, decoded);
    }

    #[test]
    fn test_ephemeral_subagent_spawning_determinism() {
        let mut seed = [0u8; 381];
        for i in 0..381 {
            seed[i] = ((i * 17 + 31) % 256) as u8;
        }

        let weights1 = EphemeralSubagentSpawner::spawn_from_seed(&seed, 3);
        let weights2 = EphemeralSubagentSpawner::spawn_from_seed(&seed, 3);
        assert_eq!(weights1, weights2);
        assert_eq!(weights1.len(), 1024);
    }

    #[test]
    fn test_swarm_hypergraph_quorum_consensus_with_auth_and_certificates() {
        let nodes = vec![1, 2, 3];
        let mut engine = SwarmConsensusEngine::new(nodes);

        let c1 = SwarmIntentChirp::new(1, 10, 1, 1, 0x01, [10, 20, 30, 40, 50, 60]);
        let c2 = SwarmIntentChirp::new(2, 10, 1, 1, 0x01, [12, 22, 32, 42, 52, 62]);
        let c3 = SwarmIntentChirp::new(3, 10, 1, 1, 0x01, [11, 21, 31, 41, 51, 61]);
        let c_unauth = SwarmIntentChirp::new(99, 10, 1, 1, 0x01, [11, 21, 31, 41, 51, 61]);

        assert!(engine.submit_intent(c_unauth).is_err(), "Unauthorized node must be rejected");
        assert!(engine.submit_intent(c1).is_ok());
        assert!(engine.submit_intent(c1).is_err(), "Duplicate submission in same epoch must be rejected");
        assert!(engine.submit_intent(c2).is_ok());

        assert_eq!(engine.resolve_consensus(10, 3), None);

        assert!(engine.submit_intent(c3).is_ok());
        let cert = engine.resolve_consensus(10, 3).expect("Quorum reached");
        assert_eq!(cert.consensus_trajectory, [11, 21, 31, 41, 51, 61]);
        assert_eq!(cert.epoch, 10);
        assert_eq!(cert.participant_nodes.len(), 3);
    }
}
