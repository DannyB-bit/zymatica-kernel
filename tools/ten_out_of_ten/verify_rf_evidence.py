#!/usr/bin/env python3
# Copyright © 2026 Zymatica
# SPDX-License-Identifier: LicenseRef-Zymatica-Covenant-2.0
# See LICENSE for terms.
"""
Fail-Closed Physical RF Telemetry & Provenance Verifier

Strictly audits existing, immutable physical RF captures in evidence/rf/phase16/:
1. Strictly fails if tx_payload.bin, rx_capture.bin, or metadata.json are missing. Never synthesizes or manufactures evidence.
2. Strips gateway framing from the recorded receiver binary (rx_capture.bin).
3. Verifies CRC-16 integrity over the reconstructed packet.
4. Decodes 3-byte Cuneiform-U telemetry radical coordinates.
5. Computes and checks BN254 MiMC-7 nullifier hash.
6. Asserts bit-exact SHA-256 identity between reconstructed RX payload and committed TX canonical bytes.
7. Emits structured report to evidence/10_00/latest/rf_phase16.json.
"""

from __future__ import annotations

import hashlib
import json
import struct
import sys
from pathlib import Path

sys.stdout.reconfigure(encoding="utf-8")


def crc16_ccitt(data: bytes) -> int:
    crc = 0xFFFF
    for byte in data:
        crc ^= (byte << 8)
        for _ in range(8):
            if crc & 0x8000:
                crc = ((crc << 1) ^ 0x1021) & 0xFFFF
            else:
                crc = (crc << 1) & 0xFFFF
    return crc


def mimc7_hash(val: int, key: int = 0, rounds: int = 91) -> int:
    q = 21888242871839275222246405745257275088548364400416034343698204186575808495617
    res = 0
    c = 0x2f8b57cf6e94
    for r in range(rounds):
        t = (val + key + (c * (r + 1))) % q
        res = pow(t, 7, q)
        val = res
    return (res + key) % q


def verify_rf_bundle() -> dict:
    print("=" * 80)
    print("  ZYMATICA PHYSICAL RF EVIDENCE PROVENANCE & SELF-RECONSTRUCTION GATE")
    print("  Mode: STRICT FAIL-CLOSED (Zero synthetic fallback)")
    print("=" * 80)

    rf_dir = Path("evidence/rf/phase16")
    meta_file = rf_dir / "metadata.json"
    tx_file = rf_dir / "tx_payload.bin"
    rx_file = rf_dir / "rx_capture.bin"
    log_file = rf_dir / "serial_telemetry.log"

    # Strict fail-closed checks
    missing = []
    if not rf_dir.is_dir():
        missing.append(str(rf_dir))
    if not meta_file.is_file():
        missing.append(str(meta_file))
    if not tx_file.is_file():
        missing.append(str(tx_file))
    if not rx_file.is_file():
        missing.append(str(rx_file))
    if not log_file.is_file():
        missing.append(str(log_file))

    if missing:
        print(f"[-] CRITICAL FAIL: Missing mandatory physical RF evidence artifacts: {missing}", file=sys.stderr)
        return {"status": "FAIL", "missing_artifacts": missing}

    metadata = json.loads(meta_file.read_text(encoding="utf-8"))
    tx_bytes = tx_file.read_bytes()
    rx_raw = rx_file.read_bytes()

    if len(tx_bytes) == 0 or len(rx_raw) == 0:
        print("[-] CRITICAL FAIL: RF binary capture files are empty", file=sys.stderr)
        return {"status": "FAIL", "error": "Empty capture files"}

    print(f"[+] Loaded RF Metadata: Experiment '{metadata['experiment_id']}' ({metadata['timestamp_utc']})")
    print(f"    - Transmitter: {metadata['hardware']['transmitter']['model']} ({metadata['radio_parameters']['carrier_frequency_hz'] / 1e6:.3f} MHz, SF={metadata['radio_parameters']['spreading_factor']})")
    print(f"    - Receiver:    {metadata['hardware']['receiver']['model']}")
    print(f"    - Distance:    {metadata['radio_parameters']['physical_separation_meters']}m line-of-sight")

    # Step 1: Strip RX Gateway Framing
    rx_header = rx_raw[:4]
    reconstructed_tx = rx_raw[4:]
    pkt_len = rx_header[3]
    if pkt_len != len(reconstructed_tx):
        print(f"[-] Gateway packet length header mismatch: {pkt_len} != {len(reconstructed_tx)}", file=sys.stderr)
        return {"status": "FAIL", "error": "Framing length mismatch"}

    # Step 2: Validate CRC-16 Checksum
    payload_body = reconstructed_tx[:-2]
    expected_crc = struct.unpack(">H", reconstructed_tx[-2:])[0]
    computed_crc = crc16_ccitt(payload_body)
    crc_valid = (expected_crc == computed_crc)
    print(f"[+] Reconstructed Packet from Raw RX Capture: {len(reconstructed_tx)} bytes")
    print(f"[+] Hardware CRC-16 CCITT: Computed=0x{computed_crc:04X}, Expected=0x{expected_crc:04X} -> {'VALID' if crc_valid else 'INVALID'}")
    if not crc_valid:
        return {"status": "FAIL", "error": "CRC-16 validation failure"}

    # Step 3: Parse Payload Header & Cuneiform Radical
    magic = payload_body[:5]
    if magic != b"ZYM10":
        print(f"[-] Invalid packet magic header: {magic}", file=sys.stderr)
        return {"status": "FAIL", "error": "Invalid magic header"}

    cuneiform_b = payload_body[5:8]
    c0 = (cuneiform_b[0] >> 4) & 0x0F
    c1 = cuneiform_b[0] & 0x0F
    c2 = (cuneiform_b[1] >> 4) & 0x0F
    c3 = cuneiform_b[1] & 0x0F
    c4 = (cuneiform_b[2] >> 4) & 0x0F
    c5 = cuneiform_b[2] & 0x0F
    coord = (c0, c1, c2, c3, c4, c5)
    print(f"[+] Decoded Cuneiform 6D Radical: {coord} (3 bytes: {cuneiform_b.hex()})")

    # Step 4: Validate MiMC-7 Nullifier
    nullifier_bytes = payload_body[8:40]
    nullifier_int = int.from_bytes(nullifier_bytes, byteorder="big")
    priv_key = 0x981247fa188e7b
    nonce = 0x140a7
    expected_nullifier = mimc7_hash(priv_key + nonce, 0)
    nullifier_valid = (nullifier_int == expected_nullifier)
    print(f"[+] MiMC-7 BN254 Nullifier Hash: 0x{nullifier_int:016x} -> {'VERIFIED' if nullifier_valid else 'INVALID'}")
    if not nullifier_valid:
        return {"status": "FAIL", "error": "Nullifier mismatch"}

    # Step 5: Bit-exact SHA-256 Checksum Matching against TX Canonical
    tx_sha256 = hashlib.sha256(tx_bytes).hexdigest()
    rx_recon_sha256 = hashlib.sha256(reconstructed_tx).hexdigest()
    exact_match = (tx_sha256 == rx_recon_sha256)
    print(f"[+] Canonical TX SHA-256:        {tx_sha256}")
    print(f"[+] Reconstructed RX SHA-256:    {rx_recon_sha256}")
    print(f"[+] Bit-Exact Provenance Chain:  {'PASS (100% BIT-EXACT)' if exact_match else 'FAIL'}")
    if not exact_match:
        return {"status": "FAIL", "error": "Reconstructed payload does not match canonical TX"}

    report = {
        "status": "PASS",
        "experiment_id": metadata["experiment_id"],
        "timestamp_utc": metadata["timestamp_utc"],
        "frequency_mhz": metadata["radio_parameters"]["carrier_frequency_hz"] / 1e6,
        "rssi_dbm": metadata["measured_rf_metrics"]["rssi_dbm"],
        "snr_db": metadata["measured_rf_metrics"]["snr_db"],
        "raw_rx_bytes": len(rx_raw),
        "reconstructed_payload_bytes": len(reconstructed_tx),
        "crc16_valid": crc_valid,
        "cuneiform_coordinates": coord,
        "mimc7_nullifier_hex": f"0x{nullifier_int:064x}",
        "tx_sha256": tx_sha256,
        "rx_reconstructed_sha256": rx_recon_sha256,
        "bit_exact_reconstruction": exact_match,
    }

    out_file = Path("evidence/10_00/latest/rf_phase16.json")
    out_file.parent.mkdir(parents=True, exist_ok=True)
    out_file.write_text(json.dumps(report, indent=2) + "\n", encoding="utf-8")
    print(f"[+] Report written to {out_file}")
    return report


if __name__ == "__main__":
    rep = verify_rf_bundle()
    if rep["status"] != "PASS":
        sys.exit(1)
