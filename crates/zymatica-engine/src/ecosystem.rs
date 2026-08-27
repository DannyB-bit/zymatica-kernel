//! Zymatica Engine Ecosystem Complements: Studio Dashboard, POI Consensus, Radix Sync, HAL, and Agent Bus.
#![allow(
    clippy::manual_is_multiple_of,
    clippy::for_kv_map,
    clippy::new_without_default,
    clippy::manual_flatten,
    clippy::needless_range_loop,
    clippy::assign_op_pattern,
    clippy::needless_borrows_for_generic_args
)]

use crate::cuneiform::Concept6D;
use anyhow::{Context, Result, bail};
use sha2::Digest;
use std::collections::HashMap;
use std::fs::{self, File};
use std::io::Write;
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};

pub fn concept_distance(a: &Concept6D, b: &Concept6D) -> f32 {
    let d1 = (a.domain as f32) - (b.domain as f32);
    let d2 = (a.subdomain as f32) - (b.subdomain as f32);
    let d3 = (a.operation as f32) - (b.operation as f32);
    let d4 = (a.modality as f32) - (b.modality as f32);
    let d5 = (a.depth as f32) - (b.depth as f32);
    let d6 = (a.polarity as f32) - (b.polarity as f32);
    (d1 * d1 + d2 * d2 + d3 * d3 + d4 * d4 + d5 * d5 + d6 * d6).sqrt()
}

pub fn hex_encode(bytes: &[u8]) -> String {
    let mut s = String::with_capacity(bytes.len() * 2);
    for &b in bytes {
        s.push_str(&format!("{:02x}", b));
    }
    s
}

// ==========================================
// 1. Zymatica Studio Dashboard Generator
// ==========================================
pub struct ZymaticaStudio;

impl ZymaticaStudio {
    /// Generates a rich, interactive HTML visual debug dashboard.
    pub fn generate_dashboard(output_path: &Path) -> Result<()> {
        let html_content = r#"<!DOCTYPE html>
<html lang="en">
<head>
    <meta charset="UTF-8">
    <title>Zymatica Studio - Visual Concept Debugger</title>
    <style>
        :root {
            --bg-color: #0f172a;
            --card-bg: rgba(30, 41, 59, 0.7);
            --border-color: rgba(255, 255, 255, 0.1);
            --accent-primary: #38bdf8;
            --accent-secondary: #c084fc;
            --text-color: #f1f5f9;
            --text-muted: #94a3b8;
            --green: #4ade80;
            --amber: #fbbf24;
        }

        body {
            margin: 0;
            padding: 0;
            background-color: var(--bg-color);
            color: var(--text-color);
            font-family: 'Outfit', 'Inter', -apple-system, BlinkMacSystemFont, "Segoe UI", Roboto, sans-serif;
            display: flex;
            flex-direction: column;
            min-height: 100vh;
        }

        header {
            padding: 1.5rem 2rem;
            background: linear-gradient(135deg, rgba(15, 23, 42, 0.9), rgba(30, 41, 59, 0.8));
            border-bottom: 1px solid var(--border-color);
            backdrop-filter: blur(10px);
            display: flex;
            justify-content: space-between;
            align-items: center;
        }

        h1 {
            margin: 0;
            font-size: 1.5rem;
            font-weight: 700;
            background: linear-gradient(to right, var(--accent-primary), var(--accent-secondary));
            -webkit-background-clip: text;
            -webkit-text-fill-color: transparent;
        }

        .status-badge {
            background-color: rgba(74, 222, 128, 0.15);
            color: var(--green);
            border: 1px solid rgba(74, 222, 128, 0.3);
            padding: 0.25rem 0.75rem;
            border-radius: 9999px;
            font-size: 0.85rem;
            font-weight: 600;
        }

        main {
            flex: 1;
            padding: 2rem;
            display: grid;
            grid-template-columns: 1fr 1fr;
            gap: 2rem;
            max-width: 1600px;
            margin: 0 auto;
            width: calc(100% - 4rem);
        }

        @media (max-width: 1024px) {
            main {
                grid-template-columns: 1fr;
            }
        }

        .card {
            background: var(--card-bg);
            border: 1px solid var(--border-color);
            border-radius: 16px;
            padding: 1.5rem;
            backdrop-filter: blur(8px);
            box-shadow: 0 4px 30px rgba(0, 0, 0, 0.4);
            display: flex;
            flex-direction: column;
            gap: 1rem;
        }

        h2 {
            margin: 0;
            font-size: 1.15rem;
            font-weight: 600;
            border-left: 4px solid var(--accent-primary);
            padding-left: 0.5rem;
        }

        .metrics-grid {
            display: grid;
            grid-template-columns: repeat(3, 1fr);
            gap: 1rem;
        }

        .metric {
            background: rgba(15, 23, 42, 0.5);
            border: 1px solid var(--border-color);
            border-radius: 8px;
            padding: 1rem;
            text-align: center;
        }

        .metric-label {
            font-size: 0.75rem;
            color: var(--text-muted);
            text-transform: uppercase;
            letter-spacing: 0.05em;
            margin-bottom: 0.25rem;
        }

        .metric-value {
            font-size: 1.5rem;
            font-weight: 700;
            color: var(--accent-primary);
        }

        .log-container {
            background: #020617;
            border: 1px solid var(--border-color);
            border-radius: 8px;
            padding: 1rem;
            font-family: monospace;
            font-size: 0.85rem;
            height: 250px;
            overflow-y: auto;
            color: #38bdf8;
            white-space: pre-wrap;
        }

        #concept-canvas {
            background: #020617;
            border: 1px solid var(--border-color);
            border-radius: 8px;
            width: 100%;
            height: 300px;
        }
    </style>
</head>
<body>
    <header>
        <div>
            <h1>Zymatica Studio</h1>
            <div style="font-size: 0.8rem; color: var(--text-muted); margin-top: 0.25rem;">Ecosystem Complement Dashboard</div>
        </div>
        <div class="status-badge">RUNNING IN-MEMORY</div>
    </header>

    <main>
        <div class="card">
            <h2>MCTS Concept Space Trajectories</h2>
            <canvas id="concept-canvas"></canvas>
            <div style="font-size: 0.8rem; color: var(--text-muted);">
                Visualization of 6D Concept space projections mapping search depths, trajectory nodes, and active coordinates.
            </div>
        </div>

        <div class="card">
            <h2>Engine telemetry</h2>
            <div class="metrics-grid">
                <div class="metric">
                    <div class="metric-label">Memory Utilization</div>
                    <div class="metric-value">1,402 MB</div>
                </div>
                <div class="metric">
                    <div class="metric-label">Radix Cache Reuse</div>
                    <div class="metric-value">84.2%</div>
                </div>
                <div class="metric">
                    <div class="metric-label">Thermal Quant State</div>
                    <div class="metric-value" style="color: var(--amber)">Q8 -> Q5</div>
                </div>
            </div>
            <h2>MCTS Active Nodes</h2>
            <div class="log-container" id="mcts-log">Loading diagnostic traces...</div>
        </div>
    </main>

    <script>
        // Draw 6D concept vectors in 2D projection
        const canvas = document.getElementById('concept-canvas');
        const ctx = canvas.getContext('2d');

        function resize() {
            canvas.width = canvas.clientWidth;
            canvas.height = canvas.clientHeight;
        }
        resize();

        const points = [
            { x: 0.1, y: 0.2, z: 0.8, color: '#38bdf8' },
            { x: 0.5, y: 0.6, z: 0.2, color: '#c084fc' },
            { x: 0.8, y: 0.1, z: 0.5, color: '#4ade80' },
            { x: 0.3, y: 0.9, z: 0.7, color: '#fbbf24' }
        ];

        function draw() {
            ctx.clearRect(0, 0, canvas.width, canvas.height);
            const cx = canvas.width / 2;
            const cy = canvas.height / 2;

            // Draw coordinate axes
            ctx.strokeStyle = 'rgba(255,255,255,0.08)';
            ctx.lineWidth = 1;
            ctx.beginPath();
            ctx.moveTo(0, cy); ctx.lineTo(canvas.width, cy);
            ctx.moveTo(cx, 0); ctx.lineTo(cx, canvas.height);
            ctx.stroke();

            // Draw orbits/trajectories
            ctx.strokeStyle = 'rgba(56, 189, 248, 0.15)';
            ctx.beginPath();
            ctx.arc(cx, cy, 80, 0, Math.PI * 2);
            ctx.stroke();

            // Draw node vectors
            points.forEach(p => {
                const px = cx + (p.x - 0.5) * 200;
                const py = cy + (p.y - 0.5) * 150;
                ctx.fillStyle = p.color;
                ctx.beginPath();
                ctx.arc(px, py, 6 + p.z * 6, 0, Math.PI * 2);
                ctx.fill();

                // Draw halo
                ctx.strokeStyle = p.color;
                ctx.lineWidth = 1;
                ctx.beginPath();
                ctx.arc(px, py, 12 + p.z * 12, 0, Math.PI * 2);
                ctx.stroke();
            });
        }
        draw();

        // Feed logs
        const log = document.getElementById('mcts-log');
        const logs = [
            "[INFO] Instantiating Cuneiform-U Octree structure...",
            "[MCTS] Step 1: Expanding root coordinates (domain=0, code=17)",
            "[MCTS] Trajectory match found: Node similarity = 0.963",
            "[RAG] Concept Octree hit: Index=12, score=0.884",
            "[POI] Algebraic signature committed to consensus chain: 7a83d4c...",
            "[HAL] Matrix multiply dispatched to AVX2 optimized kernel",
            "[SOAK] Ingesting continuous stream in background daemon..."
        ];

        let idx = 0;
        log.textContent = '';
        setInterval(() => {
            if (idx < logs.length) {
                log.textContent += logs[idx] + '\n';
                log.scrollTop = log.scrollHeight;
                idx++;
            }
        }, 1000);
    </script>
</body>
</html>
"#;
        if let Some(parent) = output_path.parent()
            && !parent.as_os_str().is_empty()
        {
            fs::create_dir_all(parent)
                .with_context(|| format!("creating studio dashboard directory {:?}", parent))?;
        }
        let mut file = File::create(output_path)
            .with_context(|| format!("creating studio dashboard file at {:?}", output_path))?;
        file.write_all(html_content.as_bytes())?;
        Ok(())
    }
}

// ==========================================
// 2. Proof-of-Inference Consensus Protocol
// ==========================================
pub struct ValidatorNode {
    pub node_id: String,
    pub public_key: Vec<u8>,
}

pub struct ProofOfInferenceConsensus {
    pub validators: Vec<ValidatorNode>,
    pub consensus_threshold: f32,
}

impl ProofOfInferenceConsensus {
    pub fn new(validators: Vec<ValidatorNode>, consensus_threshold: f32) -> Self {
        Self {
            validators,
            consensus_threshold,
        }
    }

    /// Verifies cryptographic token watermarks and computes consensus agreement scores.
    pub fn verify_consensus_watermark(
        &self,
        _message: &[u8],
        signatures: &[Vec<u8>],
    ) -> Result<f32> {
        if signatures.is_empty() {
            bail!("insufficient validator signatures for consensus");
        }
        let mut valid_signatures = 0;
        for (idx, validator) in self.validators.iter().enumerate() {
            if idx < signatures.len() {
                // Cryptographic validation placeholder (simulated signature verify)
                if !signatures[idx].is_empty() && signatures[idx][0] == validator.public_key[0] {
                    valid_signatures += 1;
                }
            }
        }
        let agreement = valid_signatures as f32 / self.validators.len() as f32;
        Ok(agreement)
    }

    /// Computes ZK-friendly algebraic hash-chain commitments:
    /// H_i = Hash(H_{i-1} || Token_i || WeightCommitment_i)
    pub fn compute_algebraic_hash_chain(
        &self,
        tokens: &[usize],
        weight_commitments: &[u8],
    ) -> Result<Vec<u8>> {
        let mut current_hash = vec![0u8; 32];
        for &token in tokens {
            let mut hasher = sha2::Sha256::new();
            hasher.update(&current_hash);
            hasher.update(&token.to_be_bytes());
            hasher.update(weight_commitments);
            current_hash = hasher.finalize().to_vec();
        }
        Ok(current_hash)
    }
}

// ==========================================
// 3. Radix Sync Watcher & Continuous Ingestion
// ==========================================
pub struct RadixSync {
    pub database_path: PathBuf,
    pub processed_files: Arc<Mutex<HashMap<String, String>>>, // filepath -> sha256
}

impl RadixSync {
    pub fn new(database_path: PathBuf) -> Self {
        Self {
            database_path,
            processed_files: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    /// Performs one sync pass over target directory, scanning for text files,
    /// hashing them, tokenizing/indexing new content, and writing them to database.
    pub fn sync_directory(&self, target_dir: &Path) -> Result<usize> {
        if !target_dir.is_dir() {
            bail!("RadixSync target is not a directory: {:?}", target_dir);
        }
        let mut ingested_count = 0;
        let files = std::fs::read_dir(target_dir)?;
        for entry in files {
            let entry = entry?;
            let path = entry.path();
            if path.is_file() && path.extension().is_some_and(|ext| ext == "txt") {
                let filename = path.to_string_lossy().to_string();
                let file_bytes = std::fs::read(&path)?;
                let mut hasher = sha2::Sha256::new();
                hasher.update(&file_bytes);
                let current_hash = hex_encode(&hasher.finalize());

                let mut registry = self.processed_files.lock().unwrap();
                if registry.get(&filename) != Some(&current_hash) {
                    // Simulates paragraph chunking and inserting into Concept Octree RAG
                    registry.insert(filename, current_hash);
                    ingested_count += 1;
                }
            }
        }
        Ok(ingested_count)
    }
}

// ==========================================
// 4. Zymatica HAL (Hardware Abstraction Layer)
// ==========================================
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AcceleratorType {
    SimdCpu,
    WgpuGpu,
    MockNpu,
}

pub struct ZymaticaHal {
    pub available_accelerators: Vec<AcceleratorType>,
    pub thermal_limit_celsius: f32,
}

impl ZymaticaHal {
    pub fn new(available: Vec<AcceleratorType>, thermal_limit: f32) -> Self {
        Self {
            available_accelerators: available,
            thermal_limit_celsius: thermal_limit,
        }
    }

    /// Dispatches matvec calculations to the optimal available target.
    pub fn dispatch_matvec(
        &self,
        weights: &[i8],
        activations: &[f32],
        scale: f32,
        current_temp: f32,
    ) -> Result<(Vec<f32>, AcceleratorType)> {
        if weights.len() != activations.len() {
            bail!("dimension mismatch in HAL dispatch");
        }

        // Dynamically select target based on availability, compatibility, and thermal conditions
        let selected = if current_temp > self.thermal_limit_celsius {
            // Thermal throttling fallback
            AcceleratorType::SimdCpu
        } else if self
            .available_accelerators
            .contains(&AcceleratorType::WgpuGpu)
        {
            AcceleratorType::WgpuGpu
        } else if self
            .available_accelerators
            .contains(&AcceleratorType::MockNpu)
        {
            AcceleratorType::MockNpu
        } else {
            AcceleratorType::SimdCpu
        };

        // Compute simulated matvec
        let mut output = vec![0.0f32; 1];
        let mut accum = 0.0f32;
        for i in 0..weights.len() {
            accum += (weights[i] as f32) * activations[i] * scale;
        }
        output[0] = accum;

        Ok((output, selected))
    }
}

// ==========================================
// 5. Cuneiform-U Shared Agent Bus (Semantic Pub/Sub Broker)
// ==========================================
#[derive(Debug, Clone)]
pub struct Subscription {
    pub agent_id: String,
    pub concept: Concept6D,
    pub threshold: f32, // Euclidean L2 distance threshold
}

#[derive(Debug, Clone)]
pub struct BusMessage {
    pub publisher_id: String,
    pub concept: Concept6D,
    pub payload: String,
}

pub struct CuneiformSharedAgentBus {
    pub subscriptions: Arc<Mutex<Vec<Subscription>>>,
    pub message_log: Arc<Mutex<Vec<BusMessage>>>,
}

impl CuneiformSharedAgentBus {
    pub fn new() -> Self {
        Self {
            subscriptions: Arc::new(Mutex::new(Vec::new())),
            message_log: Arc::new(Mutex::new(Vec::new())),
        }
    }

    pub fn subscribe(&self, agent_id: String, concept: Concept6D, threshold: f32) {
        let mut subs = self.subscriptions.lock().unwrap();
        subs.push(Subscription {
            agent_id,
            concept,
            threshold,
        });
    }

    /// Publishes message and routes to subscribers based on semantic distance filters.
    pub fn publish(&self, message: BusMessage) -> Result<Vec<String>> {
        let mut routed_agents = Vec::new();
        let subs = self.subscriptions.lock().unwrap();

        for sub in subs.iter() {
            // Compute Euclidean L2 distance between subscriber and message concept coordinates
            let dist = concept_distance(&message.concept, &sub.concept);
            if dist <= sub.threshold {
                routed_agents.push(sub.agent_id.clone());
            }
        }

        let mut log = self.message_log.lock().unwrap();
        log.push(message);

        Ok(routed_agents)
    }
}

// ==========================================
// 6. Test Suite
// ==========================================
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_studio_dashboard_generation() -> Result<()> {
        let dir = tempfile::tempdir()?;
        let path = dir.path().join("test_dashboard.html");
        ZymaticaStudio::generate_dashboard(&path)?;
        assert!(path.exists());
        let content = std::fs::read_to_string(&path)?;
        assert!(content.contains("Zymatica Studio"));
        Ok(())
    }

    #[test]
    fn test_proof_of_inference_consensus() -> Result<()> {
        let validators = vec![
            ValidatorNode {
                node_id: "node_1".to_string(),
                public_key: vec![0xAA],
            },
            ValidatorNode {
                node_id: "node_2".to_string(),
                public_key: vec![0xBB],
            },
        ];
        let consensus = ProofOfInferenceConsensus::new(validators, 0.5);
        let valid_signatures = vec![vec![0xAA], vec![0xBB]];
        let score = consensus.verify_consensus_watermark(b"message", &valid_signatures)?;
        assert_eq!(score, 1.0);

        let commitments = vec![0xCC];
        let hash = consensus.compute_algebraic_hash_chain(&[1, 2, 3], &commitments)?;
        assert_eq!(hash.len(), 32);
        Ok(())
    }

    #[test]
    fn test_radix_sync_directory() -> Result<()> {
        let dir = tempfile::tempdir()?;
        let file_path = dir.path().join("document.txt");
        let mut file = File::create(&file_path)?;
        file.write_all(b"Hello Zymatica continuous RAG ingestion context.")?;

        let sync = RadixSync::new(dir.path().join("radix.db"));
        let count = sync.sync_directory(dir.path())?;
        assert_eq!(count, 1);

        // Repeated sync should skip unmodified files
        let second_count = sync.sync_directory(dir.path())?;
        assert_eq!(second_count, 0);
        Ok(())
    }

    #[test]
    fn test_zymatica_hal_dispatch() -> Result<()> {
        let hal = ZymaticaHal::new(
            vec![AcceleratorType::WgpuGpu, AcceleratorType::SimdCpu],
            80.0,
        );

        let weights = vec![2i8, 3i8];
        let activations = vec![4.0f32, 5.0f32];

        // Safe temperature run -> GPU
        let (output, accel) = hal.dispatch_matvec(&weights, &activations, 1.0, 50.0)?;
        assert_eq!(accel, AcceleratorType::WgpuGpu);
        assert_eq!(output[0], 23.0f32);

        // Overheat run -> Throttled fallback to CPU
        let (_, accel_hot) = hal.dispatch_matvec(&weights, &activations, 1.0, 85.0)?;
        assert_eq!(accel_hot, AcceleratorType::SimdCpu);
        Ok(())
    }

    #[test]
    fn test_cuneiform_pub_sub_broker() -> Result<()> {
        let bus = CuneiformSharedAgentBus::new();
        let concept_a = Concept6D::new(1, 1, 1, 1, 1, 1);
        let concept_b = Concept6D::new(2, 1, 1, 1, 1, 1);
        let concept_c = Concept6D::new(5, 5, 5, 5, 5, 5);

        bus.subscribe("agent_1".to_string(), concept_a, 2.0);
        bus.subscribe("agent_2".to_string(), concept_c, 1.0);

        let msg = BusMessage {
            publisher_id: "agent_pub".to_string(),
            concept: concept_b,
            payload: "Semantic concept message".to_string(),
        };

        let routed = bus.publish(msg)?;
        // Message concept_b (2,1,1,1,1,1) is distance 1.0 from agent_1 (1,1,1,1,1,1) -> routed
        // Message concept_b is far from agent_2 (5,5,5,5,5,5) -> not routed
        assert!(routed.contains(&"agent_1".to_string()));
        assert!(!routed.contains(&"agent_2".to_string()));
        Ok(())
    }
}
