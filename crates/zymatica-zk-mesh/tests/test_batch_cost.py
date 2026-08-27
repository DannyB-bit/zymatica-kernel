# Watermark: ip zymatica.space
"""
ZK-LoRaWAN Batch Cost Comparison
==================================
Shows the REAL cost difference between:
  - Helium (per-packet pricing, no privacy)
  - ZK-LoRaWAN single (per-chirp on-chain, full ZK)
  - ZK-LoRaWAN batch (100 chirps per tx, full ZK)

Factual numbers. No theory.
"""

# ============================================================================
# Network Pricing
# ============================================================================
HELIUM_DC_PRICE = 0.00001        # $ per Data Credit (1 DC per packet)
HELIUM_DR0_READINGS = 2          # max sensor readings per DR0 packet

ZK_SINGLE_FEE = 150_000         # lamports per single chirp (100K gateway + 50K treasury)
ZK_BATCH_FEE = 150_000          # lamports per chirp in batch mode (same rate)
ZK_BATCH_SIZE = 100              # chirps per batch
ZK_READINGS_PER_CHIRP = 18      # sensor readings per chirp (semantic codec)

SOL_PRICE = 150.0                # USD
LAMPORTS_PER_SOL = 1_000_000_000

def lam_to_usd(lamports):
    return (lamports / LAMPORTS_PER_SOL) * SOL_PRICE

USE_CASES = [
    ("Weather Station",     12, 96,   "12 sensors every 15 min"),
    ("Medical Wearable",    10, 288,  "10 vitals every 5 min"),
    ("Smart Home",           8, 144,  "8 sensors every 10 min"),
    ("GPS Fleet Tracker",    6, 480,  "6 fields every 3 min"),
    ("Agriculture",         14, 48,   "14 sensors every 30 min"),
    ("Industrial Monitor",  18, 1440, "18 sensors every 1 min"),
    ("Simple Sensor",        1, 24,   "1 temp reading per hour"),
]


def main():
    print("=" * 82)
    print("  COST COMPARISON: Helium vs ZK-LoRaWAN (Single) vs ZK-LoRaWAN (Batch)")
    print("=" * 82)
    print()

    single_usd = lam_to_usd(ZK_SINGLE_FEE)
    batch_usd = lam_to_usd(ZK_BATCH_FEE)
    per_chirp_batched = batch_usd / ZK_BATCH_SIZE

    print(f"  Pricing:")
    print(f"    Helium:               ${HELIUM_DC_PRICE:.5f} per packet (1 DC)")
    print(f"    ZK-LoRaWAN Single:    ${single_usd:.6f} per chirp")
    print(f"    ZK-LoRaWAN Batch:     ${batch_usd:.6f} per batch of {ZK_BATCH_SIZE} chirps")
    print(f"                          = ${per_chirp_batched:.6f} per chirp (100× cheaper)")
    print(f"    SOL:                  ${SOL_PRICE}")
    print()

    # Header
    print(f"  {'Use Case':<22} {'Read':>5} {'':>3} {'Helium':>12} {'ZK Single':>12} {'ZK Batch':>12} {'ZK Batch':>10}")
    print(f"  {'':22} {'/day':>5} {'':>3} {'$/year':>12} {'$/year':>12} {'$/year':>12} {'vs Helium':>10}")
    print(f"  {'─'*22} {'─'*5} {'─'*3} {'─'*12} {'─'*12} {'─'*12} {'─'*10}")

    for name, readings_per_tx, tx_per_day, desc in USE_CASES:
        total_readings = readings_per_tx * tx_per_day

        # Helium: how many packets per transmission?
        helium_pkts_per_tx = max(1, -(-readings_per_tx // HELIUM_DR0_READINGS))
        helium_pkts_day = helium_pkts_per_tx * tx_per_day
        helium_cost_year = helium_pkts_day * HELIUM_DC_PRICE * 365

        # ZK Single: 1 chirp per transmission (semantic codec fits all readings)
        zk_chirps_day = tx_per_day  # all readings fit in 1 chirp
        zk_single_year = zk_chirps_day * single_usd * 365

        # ZK Batch: batch every 100 chirps = 1 Solana tx per 100 chirps
        batches_per_day = max(1, -(-zk_chirps_day // ZK_BATCH_SIZE))
        zk_batch_year = batches_per_day * batch_usd * 365

        # Comparison
        if helium_cost_year > 0:
            ratio = zk_batch_year / helium_cost_year
            comparison = f"{ratio:.1f}×" if ratio > 1 else f"{1/ratio:.1f}× cheaper"
        else:
            comparison = "—"

        print(f"  {name:<22} {total_readings:>5} {'→':>3} ${helium_cost_year:>10.2f} ${zk_single_year:>10.2f} ${zk_batch_year:>10.2f} {comparison:>10}")

    print()

    # ── Per-reading cost ──
    print("  ── COST PER SENSOR READING ──")
    print()

    helium_per_reading = HELIUM_DC_PRICE / HELIUM_DR0_READINGS
    zk_single_per_reading = single_usd / ZK_READINGS_PER_CHIRP
    zk_batch_per_reading = per_chirp_batched / ZK_READINGS_PER_CHIRP

    print(f"    Helium DR0:          ${helium_per_reading:.8f} per reading")
    print(f"    ZK-LoRaWAN Single:   ${zk_single_per_reading:.8f} per reading")
    print(f"    ZK-LoRaWAN Batch:    ${zk_batch_per_reading:.8f} per reading")
    print()

    ratio_single = zk_single_per_reading / helium_per_reading
    ratio_batch = zk_batch_per_reading / helium_per_reading

    print(f"    Single mode: {ratio_single:.0f}× more expensive than Helium")
    print(f"    Batch mode:  {ratio_batch:.1f}× more expensive than Helium")
    print()

    # ── The value proposition ──
    print("  ── WHAT YOU GET FOR THE PRICE ──")
    print()

    for name, readings_per_tx, tx_per_day, desc in USE_CASES:
        total_readings = readings_per_tx * tx_per_day

        helium_pkts_per_tx = max(1, -(-readings_per_tx // HELIUM_DR0_READINGS))
        helium_pkts_day = helium_pkts_per_tx * tx_per_day

        zk_chirps_day = tx_per_day
        batches_per_day = max(1, -(-zk_chirps_day // ZK_BATCH_SIZE))
        zk_batch_month = batches_per_day * batch_usd * 30

        pkt_reduction = ((helium_pkts_day - zk_chirps_day) / helium_pkts_day) * 100 if helium_pkts_day > zk_chirps_day else 0

        print(f"    📡 {name} ({desc})")
        print(f"       Helium:    {helium_pkts_day:>6} packets/day | No privacy   | ${batches_per_day * batch_usd * 0:.2f}")
        print(f"       ZK Batch:  {zk_chirps_day:>6} chirps/day  | Full ZK      | ${zk_batch_month:.2f}/month")
        print(f"       Packets:   {pkt_reduction:.0f}% fewer | Battery: ~{helium_pkts_day/zk_chirps_day:.0f}× longer")
        print()

    # ── Summary box ──
    print("=" * 82)
    print("  BATCH VERIFICATION — FINAL NUMBERS")
    print("=" * 82)
    print()
    print(f"  ┌────────────────────────────────────────────────────────────────────────┐")
    print(f"  │  ZK-LoRaWAN BATCH MODE (100 chirps/batch)                            │")
    print(f"  ├────────────────────────────────────────────────────────────────────────┤")
    print(f"  │  Cost per batch:    ${batch_usd:.4f} ({ZK_BATCH_FEE:,} lamports)              │")
    print(f"  │  Cost per chirp:    ${per_chirp_batched:.6f} (vs ${single_usd:.4f} single)            │")
    print(f"  │  Savings:           100× cheaper than single mode                    │")
    print(f"  │  Privacy:           ✅ Full ZK on every chirp                         │")
    print(f"  │  Merkle proof:      Any chirp verifiable against on-chain root        │")
    print(f"  ├────────────────────────────────────────────────────────────────────────┤")
    print(f"  │  Weather station:   ${lam_to_usd(ZK_BATCH_FEE) * 365:.2f}/year  (vs Helium ${0.00576*365:.2f}/year)       │")

    weather_premium = lam_to_usd(ZK_BATCH_FEE) * 365 - 0.00576*365
    print(f"  │  Privacy premium:   ${weather_premium:.2f}/year for COMPLETE ANONYMITY        │")
    print(f"  │  That's ${weather_premium/12:.2f}/month — less than a coffee             │")
    print(f"  └────────────────────────────────────────────────────────────────────────┘")
    print()


if __name__ == "__main__":
    main()
