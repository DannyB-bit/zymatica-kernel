# Watermark: ip zymatica.space
"""
Helium vs ZK-LoRaWAN Cost Comparison
======================================
Calculates real costs for typical IoT use cases.
Shows packets needed, daily cost, and yearly cost.
"""

import json

# ============================================================================
# Network Parameters
# ============================================================================

# Helium (standard LoRaWAN)
HELIUM_DR0_PAYLOAD = 11     # bytes app payload at DR0 (max range)
HELIUM_DR1_PAYLOAD = 53     # bytes at DR1
HELIUM_DC_PER_PACKET = 1    # 1 Data Credit per uplink (up to 24 bytes)
HELIUM_DC_PRICE_USD = 0.00001  # $0.00001 per Data Credit (fixed by HNT burn)

# ZK-LoRaWAN (semantic compression + full ZK privacy)
ZK_SEMANTIC_PAYLOAD = 57    # bytes available in ZK pipe
ZK_READINGS_PER_CHIRP_U8 = 18   # U8 sensor values per chirp
ZK_READINGS_PER_CHIRP_F16 = 14  # F16 sensor values per chirp
ZK_LAMPORTS_PER_PACKET = 150_000  # total fee (100K gateway + 50K treasury)
SOL_PRICE_USD = 150.0       # current SOL price
ZK_COST_PER_PACKET_USD = (ZK_LAMPORTS_PER_PACKET / 1_000_000_000) * SOL_PRICE_USD

# How many sensor readings fit in each network per packet
# Helium DR0: 11 bytes. A typical sensor reading is 4-6 bytes (ID + value)
# Conservatively: 2 readings per Helium DR0 packet
HELIUM_DR0_READINGS_PER_PACKET = 2
HELIUM_DR1_READINGS_PER_PACKET = 10

# ============================================================================
# IoT Use Cases
# ============================================================================
USE_CASES = [
    {
        "name": "Weather Station",
        "readings_per_transmission": 12,
        "transmissions_per_day": 96,  # every 15 minutes
        "description": "12 sensors (temp, humidity, pressure, wind, UV, rain, etc.)",
    },
    {
        "name": "Medical Wearable",
        "readings_per_transmission": 10,
        "transmissions_per_day": 288,  # every 5 minutes
        "description": "10 vitals (HR, SpO2, BP, temp, steps, etc.)",
    },
    {
        "name": "Smart Home Security",
        "readings_per_transmission": 8,
        "transmissions_per_day": 144,  # every 10 minutes
        "description": "8 sensors (motion, door, lock, camera, temp, humidity, CO2, battery)",
    },
    {
        "name": "GPS Fleet Tracker",
        "readings_per_transmission": 6,
        "transmissions_per_day": 480,  # every 3 minutes
        "description": "6 fields (lat, lon, speed, heading, altitude, battery)",
    },
    {
        "name": "Agricultural Sensor Array",
        "readings_per_transmission": 14,
        "transmissions_per_day": 48,  # every 30 minutes
        "description": "14 readings (soil pH, NPK, moisture, temp, humidity, etc.)",
    },
    {
        "name": "Industrial Monitor",
        "readings_per_transmission": 18,
        "transmissions_per_day": 1440,  # every 1 minute
        "description": "18 sensors (voltage, current, freq, power, temps, vibration)",
    },
    {
        "name": "Simple Sensor (temp only)",
        "readings_per_transmission": 1,
        "transmissions_per_day": 24,  # hourly
        "description": "1 temperature reading per hour",
    },
]


def main():
    print("=" * 80)
    print("  HELIUM vs ZK-LoRaWAN — COST COMPARISON")
    print("  Factual. Per use case. Daily and yearly costs.")
    print("=" * 80)
    print()
    print(f"  Network Parameters:")
    print(f"    Helium DR0 payload:     {HELIUM_DR0_PAYLOAD} bytes ({HELIUM_DR0_READINGS_PER_PACKET} readings/packet)")
    print(f"    Helium DC cost:         ${HELIUM_DC_PRICE_USD} per packet")
    print(f"    ZK-LoRaWAN payload:     {ZK_SEMANTIC_PAYLOAD} bytes ({ZK_READINGS_PER_CHIRP_U8} readings/chirp)")
    print(f"    ZK-LoRaWAN fee:         {ZK_LAMPORTS_PER_PACKET:,} lamports (${ZK_COST_PER_PACKET_USD:.6f})")
    print(f"    SOL price:              ${SOL_PRICE_USD}")
    print()

    # Header
    print(f"  {'─' * 76}")
    print(f"  {'Use Case':<24} {'Readings':>8} {'':>4} {'Helium DR0':>16} {'ZK-LoRaWAN':>16} {'Savings':>8}")
    print(f"  {'':24} {'/day':>8} {'':>4} {'pkts    cost':>16} {'chirps  cost':>16} {'':>8}")
    print(f"  {'─' * 76}")

    total_helium_yearly = 0
    total_zk_yearly = 0

    results = []

    for uc in USE_CASES:
        total_readings = uc["readings_per_transmission"] * uc["transmissions_per_day"]

        # Helium: how many packets needed?
        packets_per_tx = max(1, -(-uc["readings_per_transmission"] // HELIUM_DR0_READINGS_PER_PACKET))  # ceiling division
        helium_packets_day = packets_per_tx * uc["transmissions_per_day"]
        helium_cost_day = helium_packets_day * HELIUM_DC_PRICE_USD
        helium_cost_year = helium_cost_day * 365

        # ZK-LoRaWAN: how many chirps needed?
        chirps_per_tx = max(1, -(-uc["readings_per_transmission"] // ZK_READINGS_PER_CHIRP_U8))
        zk_chirps_day = chirps_per_tx * uc["transmissions_per_day"]
        zk_cost_day = zk_chirps_day * ZK_COST_PER_PACKET_USD
        zk_cost_year = zk_cost_day * 365

        # Packet reduction
        packet_reduction = ((helium_packets_day - zk_chirps_day) / helium_packets_day) * 100 if helium_packets_day > 0 else 0

        total_helium_yearly += helium_cost_year
        total_zk_yearly += zk_cost_year

        results.append({
            "name": uc["name"],
            "desc": uc["description"],
            "readings_day": total_readings,
            "helium_packets": helium_packets_day,
            "helium_cost_day": helium_cost_day,
            "helium_cost_year": helium_cost_year,
            "zk_chirps": zk_chirps_day,
            "zk_cost_day": zk_cost_day,
            "zk_cost_year": zk_cost_year,
            "packet_reduction": packet_reduction,
        })

        print(f"  {uc['name']:<24} {total_readings:>6}   →  {helium_packets_day:>5} ${helium_cost_day:>8.5f}  {zk_chirps_day:>5}  ${zk_cost_day:>7.4f}  {packet_reduction:>5.0f}%↓")

    print(f"  {'─' * 76}")
    print()

    # ── Detailed breakdown ──
    print("  ── DETAILED BREAKDOWN ──")
    print()

    for r in results:
        print(f"  📡 {r['name']} — {r['desc']}")
        print(f"     Readings/day: {r['readings_day']:,}")
        print()
        print(f"     {'':20} {'Helium DR0':>20} {'ZK-LoRaWAN':>20}")
        print(f"     {'Packets/day':20} {r['helium_packets']:>20,} {r['zk_chirps']:>20,}")
        print(f"     {'Cost/day':20} {'${:,.6f}'.format(r['helium_cost_day']):>20} {'${:,.6f}'.format(r['zk_cost_day']):>20}")
        print(f"     {'Cost/month':20} {'${:,.4f}'.format(r['helium_cost_day']*30):>20} {'${:,.4f}'.format(r['zk_cost_day']*30):>20}")
        print(f"     {'Cost/year':20} {'${:,.4f}'.format(r['helium_cost_year']):>20} {'${:,.4f}'.format(r['zk_cost_year']):>20}")
        print(f"     {'Packet reduction':20} {'—':>20} {'{:.0f}% fewer packets'.format(r['packet_reduction']):>20}")
        print(f"     {'Privacy':20} {'❌ None':>20} {'✅ Full ZK':>20}")
        print()

    # ── The real comparison ──
    print("=" * 80)
    print("  THE REAL COMPARISON")
    print("=" * 80)
    print()
    print(f"  Helium is cheaper per packet:  ${HELIUM_DC_PRICE_USD} vs ${ZK_COST_PER_PACKET_USD:.6f}")
    print(f"  But ZK-LoRaWAN sends {ZK_READINGS_PER_CHIRP_U8}× more data per packet.")
    print()
    print(f"  Effective cost per READING:")
    print(f"    Helium DR0:    ${HELIUM_DC_PRICE_USD / HELIUM_DR0_READINGS_PER_PACKET:.7f} per reading (2 readings/packet)")
    print(f"    ZK-LoRaWAN:    ${ZK_COST_PER_PACKET_USD / ZK_READINGS_PER_CHIRP_U8:.7f} per reading (18 readings/chirp)")
    print()

    helium_per_reading = HELIUM_DC_PRICE_USD / HELIUM_DR0_READINGS_PER_PACKET
    zk_per_reading = ZK_COST_PER_PACKET_USD / ZK_READINGS_PER_CHIRP_U8

    if zk_per_reading > helium_per_reading:
        ratio = zk_per_reading / helium_per_reading
        print(f"  ZK-LoRaWAN costs {ratio:.0f}× more per reading than Helium.")
        print(f"  BUT: You get FULL ZERO-KNOWLEDGE PRIVACY.")
        print(f"  Helium exposes: DevEUI, DevAddr, gateway ID, location, timing.")
        print(f"  ZK-LoRaWAN exposes: NOTHING.")
    else:
        print(f"  ZK-LoRaWAN is CHEAPER per reading than Helium!")

    print()
    print(f"  ┌────────────────────────────────────────────────────────────────┐")
    print(f"  │  WHAT THE CLIENT PAYS FOR PRIVACY                            │")
    print(f"  ├────────────────────────────────────────────────────────────────┤")

    for r in results:
        premium = r['zk_cost_year'] - r['helium_cost_year']
        print(f"  │  {r['name']:<28} +${premium:>8.2f}/year for full ZK   │")

    print(f"  ├────────────────────────────────────────────────────────────────┤")
    print(f"  │  Average privacy premium: ~${(total_zk_yearly - total_helium_yearly)/len(results):>6.2f}/year per device   │")
    print(f"  │  That's ${(total_zk_yearly - total_helium_yearly)/len(results)/12:>6.2f}/month for COMPLETE ANONYMITY       │")
    print(f"  └────────────────────────────────────────────────────────────────┘")
    print()

    # ── Packet efficiency ──
    print(f"  ── PACKET EFFICIENCY ──")
    print()
    print(f"  A weather station sending 12 readings every 15 minutes:")
    print(f"    Helium DR0:   6 packets × 96 times = {6*96:,} packets/day")
    print(f"    ZK-LoRaWAN:   1 chirp   × 96 times =  {1*96:,} packets/day")
    print(f"    Reduction:    {((6*96 - 96) / (6*96)) * 100:.0f}% fewer transmissions")
    print(f"    Battery life: ~6× longer (fewer radio transmissions)")
    print()
    print(f"  An industrial monitor sending 18 readings every minute:")
    print(f"    Helium DR0:   9 packets × 1440 times = {9*1440:,} packets/day")
    print(f"    ZK-LoRaWAN:   1 chirp   × 1440 times =  {1*1440:,} packets/day")
    print(f"    Reduction:    {((9*1440 - 1440) / (9*1440)) * 100:.0f}% fewer transmissions")
    print(f"    Battery life: ~9× longer")
    print()


if __name__ == "__main__":
    main()
