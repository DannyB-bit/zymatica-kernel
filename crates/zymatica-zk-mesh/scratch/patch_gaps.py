import re

def main():
    # 1. Read compile_pdf.py
    with open("scratch/compile_pdf.py", "r", encoding="utf-8") as f:
        code = f.read()

    # 2. Patch title page subtitle to reference Language-U Protocol
    code = re.sub(
        r"A Solana-Style ZK-Compressed Shielded Pool and Lossless Proof Compression Layer for LoRaWAN Communications",
        "A Solana-Style ZK-Compressed Shielded Pool and Lossless Proof Compression Layer (Language-U Protocol)",
        code
    )

    # 3. Patch Page 5 System Architecture to insert 6D Cuneiform-U Coordinates section
    cuneiform_insert = """story.append(Paragraph("3. System Architecture", h1_style))
    story.append(Spacer(1, 10))
    story.append(get_system_topology_diagram())
    story.append(Spacer(1, 10))
    story.append(Paragraph("3.1 6D Cuneiform-U Semantic Coordinates", h2_style))
    story.append(Paragraph(
        "Under Component 02 of the Language-U protocol, edge nodes compress message intent into a 6-axis semantic coordinate "
        "system representing: Domain, Subdomain, Modality, Polarity, Strength, and Depth. These coordinates are committed "
        "using Pedersen commitments (C = g^v * h^r mod BN254) to enable private gating verification at the RF layer without "
        "disclosing the node's exact data values or geographical location.", normal_style
    ))
    story.append(Spacer(1, 5))"""

    code = re.sub(
        r'story\.append\(Paragraph\("3\. System Architecture", h1_style\)\)\n\s+story\.append\(Spacer\(1, 10\)\)\n\s+story\.append\(get_system_topology_diagram\(\)\)\n\s+story\.append\(Spacer\(1, 10\)\)',
        cuneiform_insert,
        code
    )

    # 4. Patch Page 10 to add microByte JIT Verifying Key Compression section
    jit_insert = """story.append(Paragraph("8. Edge Prover-Gateway Division", h1_style))
    story.append(Paragraph(
        "To understand how ZK-LoRaWAN scale-out works, it is essential to clarify the division of labor between the Prover "
        "(the edge node/device) and the Verifier (the Solana validator network):", normal_style
    ))
    story.append(Paragraph("8.1 microByte JIT Verifying Key Compression", h2_style))
    story.append(Paragraph(
        "For flash-constrained edge nodes (such as ESP32 microcontrollers), storing standard Groth16 verifying keys (~2-4 KB) "
        "wastes precious storage. Under Component 19, ZK-LoRaWAN implements microByte JIT VK compression. Verifying keys are "
        "compressed into compact seeds (<1 KB) and dynamically inflated in-memory on the edge device during verification time, "
        "optimizing system storage constraints.", normal_style
    ))
    story.append(Spacer(1, 5))"""

    code = re.sub(
        r'story\.append\(Paragraph\("8\. Edge Prover-Gateway Division", h1_style\)\)\n\s+story\.append\(Paragraph\(\n\s+"To understand how ZK-LoRaWAN scale-out works, it is essential to clarify the division of labor between the Prover "\n\s+"\(the edge node/device\) and the Verifier \(the Solana validator network\):", normal_style\n\s+\)\)',
        jit_insert,
        code
    )

    # 5. Patch Page 13 to insert XOR-FEC Parity Shell section
    fec_insert = """story.append(Paragraph("11. Performance & Bandwidth Analysis", h1_style))
    story.append(Paragraph(
        "Because LoRa is a low-bandwidth modulation scheme operating in unlicensed Industrial, Scientific, and Medical "
        "(ISM) radio bands, packet size and regulatory compliance are critical. ZK-LoRaWAN operates on license-free "
        "spectrum globally, including US915 (902-928 MHz) in North America, EU868 (863-870 MHz) in Europe (subject to "
        "a strict 1% duty cycle limit), and AU915 in South America. This allows completely permissionless deployment "
        "with typical transmission ranges of 2 to 5 km in urban areas, 10 to 15 km in rural line-of-sight, and up to "
        "30+ km from high-elevation nodes (such as hilltops or drones).", normal_style
    ))
    story.append(Paragraph("11.1 XOR Forward Error Correction (FEC) Parity Shell", h2_style))
    story.append(Paragraph(
        "Cryptographic proofs are highly brittle: a single corrupted bit over the air invalidates the entire Groth16 verification, "
        "draining edge batteries. Under Component 06, ZK-LoRaWAN wraps compressed proofs in an XOR-FEC parity shell. If up to 20% "
        "of the packet is lost or corrupted over the air, the gateway relayer reconstructs the proof locally without forcing "
        "a costly retransmission.", normal_style
    ))
    story.append(Spacer(1, 5))"""

    code = re.sub(
        r'story\.append\(Paragraph\("11\. Performance & Bandwidth Analysis", h1_style\)\)\n\s+story\.append\(Paragraph\(\n\s+"Because LoRa is a low-bandwidth modulation scheme operating.*?\n\s+30\+ km from high-elevation nodes \(such as hilltops or drones\)\.", normal_style\n\s+\)\)',
        fec_insert,
        code,
        flags=re.DOTALL
    )

    # 6. Correct curve references on Page 16 (Section 14.2)
    code = re.sub(
        r'natively processes 192-byte BLS12-381 compressed proofs or Pasta curve evaluations on-chain for Orchard-level security',
        'natively processes 128-byte BN254 compressed proofs on-chain for production-grade security, verifying pairing check algebra directly over the BN254 prime field',
        code
    )
    code = re.sub(
        r'Zcash-grade security',
        'production-grade security',
        code
    )

    # 7. Add Gateway Reputation updates to Roadmap (Page 17)
    reputation_insert = """story.append(Paragraph(
        "• <b>Gateway Peer Reputation:</b> Integrate peer reputation score updates using RCRA Resonance Alignment (exponential "
        "moving average updates) committed via Pedersen range proofs on-chain.", bullet_style
    ))
    story.append(Paragraph(
        "• <b>Solana Micropayment Integration:</b> Enable automated, real-time micropayment rewards for valid mesh routing "
        "proofs, interfacing with ChirpStack and The Things Network (TTN).", bullet_style
    ))"""

    code = re.sub(
        r'story\.append\(Paragraph\(\n\s+"• <b>Solana Micropayment Integration:</b> Enable.*?\n\s+\)\)',
        reputation_insert,
        code,
        flags=re.DOTALL
    )

    # 8. Save updated compile_pdf.py
    with open("scratch/compile_pdf.py", "w", encoding="utf-8") as f:
        f.write(code)
    print("Patched compile_pdf.py successfully.")

    # 9. Read and update WHITEPAPER.md to include the exact same alignment descriptions
    with open("WHITEPAPER.md", "r", encoding="utf-8") as f:
        md = f.read()

    md = re.sub(
        r'A Solana-Style ZK-Compressed Shielded Pool and Lossless Proof Compression Layer for LoRaWAN Communications',
        'A Solana-Style ZK-Compressed Shielded Pool and Lossless Proof Compression Layer (Language-U Protocol)',
        md
    )

    # Insert cuneiform coordinates in md
    md = re.sub(
        r'## Layer 1: Elliptic Curve Identity Derivation',
        '## Layer 0.5: 6D Cuneiform-U Semantic Coordinates\nUnder Component 02 of the Language-U protocol, edge nodes compress message intent into a 6-axis semantic coordinate system representing: Domain, Subdomain, Modality, Polarity, Strength, and Depth. These coordinates are committed using Pedersen commitments (C = g^v * h^r mod BN254) to enable private gating verification at the RF layer without disclosing the node\'s exact data values or geographical location.\n\n## Layer 1: Elliptic Curve Identity Derivation',
        md
    )

    # Insert JIT VK compression in md
    md = re.sub(
        r'# 8. Edge Prover-Gateway Division\nTo understand how ZK-LoRaWAN scale-out works, it is essential to clarify the division of labor between the Prover \(the edge node/device\) and the Verifier \(the Solana validator network\):',
        '# 8. Edge Prover-Gateway Division\nTo understand how ZK-LoRaWAN scale-out works, it is essential to clarify the division of labor between the Prover (the edge node/device) and the Verifier (the Solana validator network):\n\n### 8.1 microByte JIT Verifying Key Compression\nFor flash-constrained edge nodes (such as ESP32 microcontrollers), storing standard Groth16 verifying keys (~2-4 KB) wastes precious storage. Under Component 19, ZK-LoRaWAN implements microByte JIT VK compression. Verifying keys are compressed into compact seeds (<1 KB) and dynamically inflated in-memory on the edge device during verification time, optimizing system storage constraints.',
        md
    )

    # Insert XOR-FEC in md
    md = re.sub(
        r'# 11. Performance & Bandwidth Analysis\nBecause LoRa is a low-bandwidth modulation scheme operating.*?\n30\+ km from high-elevation nodes \(such as hilltops or drones\)\.',
        lambda m: m.group(0) + '\n\n### 11.1 XOR Forward Error Correction (FEC) Parity Shell\nCryptographic proofs are highly brittle: a single corrupted bit over the air invalidates the entire Groth16 verification, draining edge batteries. Under Component 06, ZK-LoRaWAN wraps compressed proofs in an XOR-FEC parity shell. If up to 20% of the packet is lost or corrupted over the air, the gateway relayer reconstructs the proof locally without forcing a costly retransmission.',
        md,
        flags=re.DOTALL
    )

    # Correct curve in md
    md = re.sub(
        r'natively processes 192-byte BLS12-381 compressed proofs or Pasta curve evaluations on-chain for Orchard-level security',
        'natively processes 128-byte BN254 compressed proofs on-chain for production-grade security, verifying pairing check algebra directly over the BN254 prime field',
        md
    )
    md = re.sub(
        r'Zcash-grade security',
        'production-grade security',
        md
    )

    # Add reputation in md roadmap
    md = re.sub(
        r'\* \*\*Solana Micropayment Integration:\*\* Enable.*?\n',
        '* **Gateway Peer Reputation:** Integrate peer reputation score updates using RCRA Resonance Alignment (exponential moving average updates) committed via Pedersen range proofs on-chain.\n* **Solana Micropayment Integration:** Enable automated, real-time micropayment rewards for valid mesh routing proofs, interfacing with ChirpStack and The Things Network (TTN).\n',
        md
    )

    with open("WHITEPAPER.md", "w", encoding="utf-8") as f:
        f.write(md)
    print("Patched WHITEPAPER.md successfully.")

if __name__ == "__main__":
    main()
