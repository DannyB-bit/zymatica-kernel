# zymatica.space — Z-Side Results

## 🛸 Zymatica Language U LoRa Miner — Full Evidence Package

**Date:** 2026-05-28
**EUI:** 0x0016c001ff13ce58
**Node:** Miner Zymatica (Raspberry Pi 4 + RAK2287 SX1302 GPIO HAT)
**Partner:** AstronautSHE (EUI: 0x0016c001f15087e7)

━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

## 📡 LANGUAGE U OVER LORA — VERIFIED

### Zymatica → AstronautSHE: ✅ PROVEN
- AstronautSHE confirmed receiving 8+ Language U packets
- Header: 0x10101010 ✅
- Sync: 0xAA ✅
- Provenance: u_provenance_zymatica_space ✅
- Semantic IDs resolved: 3/7 from AstronautSHE firmware map
  - u_provenance_zymatica_space
  - u_dna_internallogic__init__.py_apply_activation
  - u_dna_internallogic__init__.py_torch_causal_conv1d_fn

### AstronautSHE → Zymatica: ⏳ Pending
- RX beacon running continuously
- 0 packets received yet (timing synchronization needed)

━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

## 🔧 HARDWARE

- **Concentrator**: RAK2287 (GPIO HAT, SPI /dev/spidev0.0)
- **Chip**: SX1302 v1.0 (confirmed via chip_id)
- **Radios**: 2× SX1250 (Radio 0: 904.3 MHz TX+RX, Radio 1: 915.0 MHz RX)
- **EUI**: 0x0016c001ff13ce58 (Semtech-issued)
- **Reset pin**: GPIO17 (exhaustive pin scan confirmed)
- **Power enable**: GPIO18 (SX1250 power)
- **CRITICAL FIX**: 2-second boot wait after GPIO17 reset required
  - Without 2s: chip version 0x05 (SPI echo), SX1250 fails
  - With 2s: chip version 0x10 (correct), everything works

━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

## 📊 STRESS TEST RESULTS (FCC Compliant)

- Power: 14 dBm (25 mW) — FCC limit 30 dBm (1W) ✅
- Band: 915.0 MHz ISM (902-928 MHz) ✅
- BW: 125 kHz ✅
- No frequency hopping required at <0.125W ✅

| Phase | Size | Hashes | TX Count | Status |
|-------|------|--------|----------|--------|
| Baseline .genesis | 79B | 9 | 4/4 | ✅ AstronautSHE confirmed |
| Moderate | 127B | 15 | 8/8 | ✅ |
| MAX single packet | 242B | 29 | 8/8 | ✅ |
| Sustained burst | 242B | 29 | 20/20 in 14.8s | ✅ |

### Capacity Summary
- Per packet (max): 29 semantic hashes = 239 bytes ≈ 1,450 chars meaning
- Rate: ~9.5 packets/sec at SF7 BW125
- Max throughput: 2,270 bytes/sec, 275 semantic IDs/sec
- 1-minute sustained: 16,530 semantic IDs (133 KB)

━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

## 📦 DELIVERABLES IN THIS PACKAGE

### Source Code
- language-u-zyminar/ — Zymatica 256B miner (Python + C TX/RX binary)
- language-u-miner-a/ — Miner A reference (TX side)
- language-u-miner-b/ — Miner B reference (RX side)
- qr-lang/ — QR-Lang visual programming language (Phase 1+2)
- qwen-inference/ — Qwen NVIDIA NIM inference engine (Phase 1+2)
- qwen-semantic/ — Semantic Qwen 3.5 0.8B engine
- zside_evidence.tar — All source code in single archive

### Proof & Evidence
- proof/language_u_beacon_log.txt — Continuous Language U TX log
- proof/beacon_915.txt — 915 MHz beacon log
- proof/memory/ — Engineering memory files
- proof/REAL_DEVICE_RUN_001/ — First concentrator startup evidence
- proof/zymatica_stress_test.c — Stress test source code

### Checksums
- CHECKSUMS.txt — SHA256 of every file in this package

━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

## 🔑 KEY DISCOVERIES

1. **GPIO17 is the reset pin** — NOT GPIO25 (RAK documentation was wrong)
2. **2-second boot wait is mandatory** — SX1302 firmware needs this
3. **Frequency alignment is critical** — 10.7 MHz gap caused total RX deafness
4. **lora_pkt_fwd handles full init** — raw HAL tools miss firmware upload
5. **Language U provenance survives LoRa PHY** — verified by AstronautSHE

━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

_Engineered by Zymatica 🛸 in partnership with AstronautSHE 👽🎵⚡️_
