# ZK-LoRaWAN RAK Field Trial Checklist

This checklist outlines the controlled procedures for verifying the ZK-LoRaWAN private routing system in a small-scale physical field trial using real RAK gateways and hardware edge nodes.

---

## 1. Pre-Trial Prerequisites & Hardware Setup

### Gateway Infrastructure
- [ ] **2–3 Real RAK Gateways/Miners** (e.g., RAK v2 / RAK7248 or similar Semtech SX1302 concentrator setups).
- [ ] **Stable Power Supply:** Minimum 5V/3A stable adapter or calibrated 12V solar-battery bank system.
- [ ] **Internet Backhaul:** Gateways configured with distinct connection types (e.g., Gateway 1 on Fiber/Wi-Fi, Gateway 2 on LTE hotspot) to test latency/packet jitter variance.
- [ ] **SX1302 Packet Forwarder Bindings:** Verification of the packet forwarder daemon mapping local UDP traffic (port `1700`) to the semantic gating routing gateway daemon.

### Edge Device Configurations
- [ ] **1–3 LoRaWAN End-Devices** (e.g., ESP32 or STM32 nodes).
- [ ] **Enclave Secure Element:** ATECC608A secure element properly soldered and interfaced via I2C.
- [ ] **ATECC608A Config Locking Check:** Confirm that the ATECC608A configuration zone is locked (using `atecc_config_check` tool or equivalent) to enable secure ECDSA execution and read-protection.
- [ ] **Tagged Firmware Baseline:** End-device firmware compiled from a clean, tagged Git commit hash.

### Administrator Control Station
- [ ] Local laptop/admin machine with:
  - Repository checked out at the identical Git commit.
  - Installed Solana CLI configured for `devnet`.
  - Funded Devnet administrator/fee payer wallet.
  - SSH access to each gateway to read system logs.
- [ ] **Devnet SOL Funding:** Ensure the gateway operator / fee payer wallet is pre-funded with at least 5–10 Devnet SOL to cover transaction costs for the reliability run, and that a local faucet script is available to request automatic refills.

---

## 2. Hardware and Node Registration Data

Record the following information before commencing tests:

| Parameter | Node / Gateway 1 | Node / Gateway 2 | Node / Gateway 3 |
| :--- | :--- | :--- | :--- |
| **Device/Gateway ID** | | | |
| **Model & Chipset** | | | |
| **Firmware Hash** | | | |
| **Geographic Location** | | | |
| **Backhaul Network Type** | | | |
| **Secure Element Serial (SN)** | | | |
| **Secure Element Pubkey** | | | |
| **RF Antenna Gain (dBi)** | | | |

---

## 3. Repository and Build Baseline Check

Run the following validation sequence on the control machine to ensure codebase integrity:

```bash
# 1. Format Check
cargo fmt --all -- --check

# 2. Clippy Lints
cargo clippy -p zk-lorawan-groth16 --all-targets --no-deps -- -D warnings

# 3. Workspace Rust Unit Tests
cargo test --workspace --quiet

# 4. JS/TS SDK Type Check
npm run typecheck

# 5. Full Pipeline Verifier Script
python tests/verify_all_modules.py

# 6. VK Matching Verification
python tests/verify_onchain_vk.py

# 7. Release Compile
cargo build -p zk-lorawan-groth16 --release
```

- [ ] All format, lint, and unit checks return `OK`.
- [ ] Repository status is clean: `git status --short` returns empty.
- [ ] Current commit hash is logged: `git rev-parse HEAD`.

---

## 4. Devnet Program Deployment

Deploy the tested Solana program commit to Solana Devnet:

```bash
solana program deploy target/deploy/zk_lorawan.so
```

Record:
* **Commit Hash:**
* **Program ID:**
* **Deploy Transaction Signature:**
* **Admin Wallet Address:**
* **Registry PDA Address:**
* **Shielded Pool PDA Address:**
* **Treasury Wallet Address:**
* **On-Chain Verifying Key (VK) Constants Hash:**
* **Ceremony Transcript Hash:**

### On-Chain Initialization Steps
- [ ] **ProtocolRegistry** initialized on-chain.
- [ ] **ShieldedEscrowPool** initialized on-chain.
- [ ] **Firmware Whitelist** updated with the compiled device firmware image hash.
- [ ] **Treasury Escrow** initialized and configured.
- [ ] **Genesis Escrow** funded on-chain.

---

## 5. Ceremony & Key Provenance Audit
- [ ] Run ceremony validation locally:
  ```bash
  zk_lorawan_prove ceremony verify --input <final_ceremony_params>
  ```
- [ ] Verify generated verifier logic matches on-chain layout:
  ```bash
  python tests/verify_onchain_vk.py
  ```
- [ ] Confirm genesis-only finalization check:
  - Finalizing with `N < 2` contributions must fail.

Record:
* `ceremony_transcript.json` hash:
* `proving_key.bin` hash:
* `verifying_key.bin` hash:
* Number of ceremony contributors (must be $\ge 2$):
* Contributor names/IDs:

---

## 6. Real-World RF Signal & Network Tuning (Crucial Additions)

Before launching end-to-end ZK transactions, verify the local physical RF channels:

- [ ] **Ambient Noise Floor Assessment:** Measure and log the baseline channel RSSI/SNR on the selected frequency (US915/EU868) with all nodes silenced to detect local interference.
- [ ] **NTP / SX1302 Clock Sync check:** Validate that gateways are synchronized to $\pm 100$ ms using local NTP pools or GPS PPS line. (Crucial because the pre-circuit replay protection rejects packets outside a strict $\pm 5$-second window).
- [ ] **SX1302 GPS Satellite Lock:** If testing Time-of-Flight (ToF) location checks, verify that the gateway has a valid 3D GPS satellite fix and PPS pulse lock in ChirpStack/packet-forwarder logs.
- [ ] **Devnet RPC Rate-Limiting Failover:** Configure client SDK with at least two fallback RPC endpoints (e.g., QuickNode or Helius) to prevent node transaction dropouts during rapid chunked writes.

---

## 7. Firmware Attestation Verification

For each edge device:
- [ ] Capture the attestation signature generated by the ATECC608A secure element.
- [ ] Submit the attestation report to the gateway local daemon.
- [ ] Confirm gateway validates the firmware signature.
- [ ] Verify that an unapproved firmware hash (modified code) is rejected instantly by the gateway before ZK proving is scheduled.

Record:
* Device ID:
* Firmware Image Hash:
* Attestation Payload Content:
* Secure Element Signature:
* Gateway Verification Result:

---

## 8. Physical LoRaWAN Packet Capture (No Simulators)

For every transmitted packet:
- [ ] Verify the packet was received via physical RF concentrator (Semtech SX1302/SX1303).
- [ ] Ensure simulator/mock packet pathways are completely disabled in the gateway config.
- [ ] **Multi-Gateway Packet Deduplication Check:** Verify that the relayer database or MQTT broker deduplicates incoming frames locally, or handles the expected on-chain nullifier error gracefully if Gateway A verifies the proof first and Gateway B submits the identical nullifier immediately after.
- [ ] Collect raw RF metrics from ChirpStack/daemon log:
  * Frequency (MHz):
  * Spreading Factor (SF):
  * Bandwidth (BW):
  * RSSI (dBm):
  * SNR (dB):
  * Payload Length (Bytes):

---

## 9. Positive End-to-End ZK Proving Flow

For at least **3 successful transmissions per gateway**, execute the full chunked on-chain verification pipeline:

1. **initializeProofContext**
2. **writeProofChunk(0)**
3. **writeProofChunk(1)**
4. **writeProofChunk(2)**
5. **verifyProofContext**
6. **closeProofContext**

Record:

### Transaction Signatures
- **Attempt 1 (Init / Chunk 0 / Chunk 1 / Chunk 2 / Verify / Close):**
- **Attempt 2 (Init / Chunk 0 / Chunk 1 / Chunk 2 / Verify / Close):**
- **Attempt 3 (Init / Chunk 0 / Chunk 1 / Chunk 2 / Verify / Close):**

### Transaction Size Validation (Must be < 1232 bytes)
* Init Transaction Size (bytes):
* Chunk 0 Size (bytes):
* Chunk 1 Size (bytes):
* Chunk 2 Size (bytes):
* Verify Transaction Size (bytes):
* Close Transaction Size (bytes):

### Expected Outcomes
- [ ] Solana Devnet program execution completes successfully.
- [ ] **Relayer Transaction Nonce & Retry Logic:** Confirm that the gateway daemon has transaction retry loop and nonce tracker enabled to handle blockhash expiration or network dropouts during the 6-transaction chunk flow without context desynchronization.
- [ ] Gateway account balance is credited with **100,000 lamports** per packet.
- [ ] Developer treasury account is credited with **50,000 lamports** per packet.
- [ ] On-chain nullifier PDA is generated to prevent double-spending.
- [ ] Registry transaction statistics increment correctly.

---

## 10. Negative Test Matrix on Devnet

Verify that the on-chain Solana verifier program actively blocks all fraudulent attempts. Record the failing transaction signatures:

### A. Unapproved Firmware Hash
* **Test:** Submit a proof using a firmware hash not present in the Registry whitelist.
* **Expected Error:** `InvalidAttestation` / whitelist check failure.
* **Failing Transaction Signature:**

### B. Nullifier Replay
* **Test:** Attempt to submit the same proof/nullifier hash combination twice.
* **Expected Error:** Account already exists (replayed nullifier rejected).
* **Failing Transaction Signature:**

### C. Corrupted Proof Payload
* **Test:** Flip a single byte inside the serialized G1/G2 proof data (chunk 0 or 1).
* **Expected Error:** `InvalidZeroKnowledgeProof` / bilinear pairing mismatch.
* **Failing Transaction Signature:**

### D. Bad Merkle Proof Path
* **Test:** Modify a sibling node value inside the Merkle membership proof (chunk 2).
* **Expected Error:** `InvalidMerkleProof` / root verification mismatch.
* **Failing Transaction Signature:**

### E. Unauthorized Gateway Binding
* **Test:** Bind the ZK-proof to Gateway A's public key, but submit the transaction from Gateway B.
* **Expected Error:** `UnauthorizedGateway` or `ProofContextUnauthorized`.
* **Failing Transaction Signature:**

### F. Incomplete Context Execution
* **Test:** Initialize proof context, submit chunk 0, and immediately call verify without submitting chunks 1 and 2.
* **Expected Error:** `ProofContextIncomplete`.
* **Failing Transaction Signature:**

---

## 11. Battery & Power Telemetry Audit (Crucial Addition)

Proving on low-power battery nodes is computationally expensive:

- [ ] **Proving Power Draw Peak:** Log current (mA) spike during the 1.2-second Groth16 proof generation on the edge microcontroller.
- [ ] **Microcontroller Battery Degradation:** Record battery voltage before and after generating 10 consecutive proofs to calculate safe duty-cycle limits.
- [ ] **Thermal Limits:** Log edge node temperature during continuous loop proving to prevent overheating in outdoor enclosures.

---

## 12. Reliability Run (30-60 Minutes)

Run the network continuously for 30–60 minutes under the following constraints:
* Minimum **25 physical packets** sent.
* Minimum **2 active gateways** online.
* At least **1 mobile node** transmitting from a moving vehicle or drone to test signal degradation.

Record:
* Total Packets Transmitted:
* Total Proofs Generated:
* Total Proofs Verified on Solana Devnet:
* Total Packets Dropped (RF collision/fading):
* Average RSSI/SNR during trial:
* Gateway daemon crash/restarts (must be 0):

---

## 13. Trial Evidence Bundle

At the completion of the trial, archive the following files as the final verification bundle:
1. `commit.txt` (exact git commit used)
2. `devnet-deployment.json` (program ID and escrow accounts)
3. `attestation-payloads.jsonl` (raw TEE signatures from nodes)
4. `gateway-logs/` (stderr/stdout logs from RAK gateways)
5. `lorawan-packets.csv` (RF signal properties: RSSI, SNR, frequency)
6. `successful-transactions.csv` (Devnet ZK verification signatures)
7. `negative-test-transactions.csv` (signatures of rejected fraud attempts)
8. `performance.csv` (proving time, RPC latency, and Solana compute units used)
9. `final-report.md` (summary of operational findings)
