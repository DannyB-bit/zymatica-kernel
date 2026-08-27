import os
import re

def main():
    # 1. Read the existing compile_pdf.py file
    with open("scratch/compile_pdf.py", "r", encoding="utf-8") as f:
        code = f.read()

    # 2. Add imports for ReportLab graphics shapes if not present
    import_addition = (
        "from reportlab.graphics.shapes import Drawing, Rect, String as DString, Line as DLine, Circle as DCircle\n"
    )
    if "from reportlab.graphics.shapes" not in code:
        code = re.sub(
            r"from reportlab\.pdfgen import canvas",
            "from reportlab.pdfgen import canvas\n" + import_addition,
            code
        )

    # 3. Define the vector diagram helper functions
    diagram_helpers = """
def get_system_topology_diagram():
    d = Drawing(480, 80)
    # Background for nodes
    # Node A: Edge IoT Node
    d.add(Rect(5, 15, 125, 50, fillColor=colors.HexColor("#1A202C"), strokeColor=colors.HexColor("#14F195"), strokeWidth=1, rx=4, ry=4))
    d.add(DString(20, 45, "Edge IoT Node", fontName="Helvetica-Bold", fontSize=9, fillColor=colors.HexColor("#FFFFFF")))
    d.add(DString(20, 30, "(Enclave/ATECC608)", fontName="Helvetica", fontSize=8, fillColor=colors.HexColor("#A0AEC0")))

    # Arrow 1: LoRa RF
    d.add(DLine(130, 40, 180, 40, strokeColor=colors.HexColor("#718096"), strokeWidth=1))
    d.add(DString(135, 45, "LoRa RF", fontName="Helvetica", fontSize=7, fillColor=colors.HexColor("#2B6CB0")))

    # Node B: Gateway Relayer
    d.add(Rect(180, 15, 125, 50, fillColor=colors.HexColor("#1A202C"), strokeColor=colors.HexColor("#9945FF"), strokeWidth=1, rx=4, ry=4))
    d.add(DString(195, 45, "Gateway Relayer", fontName="Helvetica-Bold", fontSize=9, fillColor=colors.HexColor("#FFFFFF")))
    d.add(DString(195, 30, "(SX1302 Concentrator)", fontName="Helvetica", fontSize=8, fillColor=colors.HexColor("#A0AEC0")))

    # Arrow 2: Solana RPC
    d.add(DLine(305, 40, 355, 40, strokeColor=colors.HexColor("#718096"), strokeWidth=1))
    d.add(DString(310, 45, "RPC Tx", fontName="Helvetica", fontSize=7, fillColor=colors.HexColor("#2B6CB0")))

    # Node C: Shielded Pool
    d.add(Rect(355, 15, 120, 50, fillColor=colors.HexColor("#1A202C"), strokeColor=colors.HexColor("#14F195"), strokeWidth=1, rx=4, ry=4))
    d.add(DString(368, 45, "Shielded Pool", fontName="Helvetica-Bold", fontSize=9, fillColor=colors.HexColor("#FFFFFF")))
    d.add(DString(368, 30, "(On-Chain Program)", fontName="Helvetica", fontSize=8, fillColor=colors.HexColor("#A0AEC0")))
    return d

def get_fee_split_diagram():
    d = Drawing(480, 110)
    # Central pool box
    d.add(Rect(15, 35, 115, 45, fillColor=colors.HexColor("#1A202C"), strokeColor=colors.HexColor("#3182CE"), strokeWidth=1.5, rx=5, ry=5))
    d.add(DString(25, 60, "Shielded Pool", fontName="Helvetica-Bold", fontSize=9, fillColor=colors.HexColor("#FFFFFF")))
    d.add(DString(25, 48, "(Global Escrow)", fontName="Helvetica", fontSize=7.5, fillColor=colors.HexColor("#A0AEC0")))

    # Split lines
    # Upper line: Gateway reward (100k lamports)
    d.add(DLine(130, 58, 220, 80, strokeColor=colors.HexColor("#14F195"), strokeWidth=1))
    d.add(Rect(220, 60, 140, 40, fillColor=colors.HexColor("#1D4ED8"), strokeColor=colors.HexColor("#14F195"), strokeWidth=1, rx=4, ry=4))
    d.add(DString(232, 82, "Gateway Relayer", fontName="Helvetica-Bold", fontSize=8.5, fillColor=colors.HexColor("#FFFFFF")))
    d.add(DString(232, 70, "+100,000 lamports (SOL)", fontName="Helvetica-Bold", fontSize=8, fillColor=colors.HexColor("#14F195")))

    # Lower line: Dev treasury (50k lamports)
    d.add(DLine(130, 58, 220, 25, strokeColor=colors.HexColor("#9945FF"), strokeWidth=1))
    d.add(Rect(220, 5, 140, 40, fillColor=colors.HexColor("#1D4ED8"), strokeColor=colors.HexColor("#9945FF"), strokeWidth=1, rx=4, ry=4))
    d.add(DString(232, 27, "Developer Treasury", fontName="Helvetica-Bold", fontSize=8.5, fillColor=colors.HexColor("#FFFFFF")))
    d.add(DString(232, 15, "+50,000 lamports (SOL)", fontName="Helvetica-Bold", fontSize=8, fillColor=colors.HexColor("#14F195")))
    return d
"""
    if "get_system_topology_diagram" not in code:
        code = re.sub(
            r"def create_whitepaper_pdf\(\):",
            diagram_helpers + "\ndef create_whitepaper_pdf():",
            code
        )

    # 4. Update Cover Page template (Page 1) to draw title, rated box, and larger 420x420 logo
    new_cover_code = """        # Page 1: Cover Page
        if self._pageNumber == 1:
            self.saveState()
            # Draw dark background
            self.setFillColor(colors.HexColor("#0d0f14"))
            self.rect(0, 0, 595.27, 841.89, fill=True, stroke=False)

            # Draw top stripe (Solana color gradient)
            self.setFillColor(colors.HexColor("#14F195"))
            self.rect(0, 826.89, 297.63, 15, fill=True, stroke=False)
            self.setFillColor(colors.HexColor("#9945FF"))
            self.rect(297.63, 826.89, 297.64, 15, fill=True, stroke=False)

            # Draw title text
            self.setFont("Helvetica-Bold", 14)
            self.setFillColor(colors.HexColor("#14F195"))
            self.drawCentredString(297.63, 720, "PROJECT: ZK-LORAWAN")

            # Draw logo image (BIGGER - 420x420)
            if os.path.exists("zk_lorawan_logo.png"):
                self.drawImage("zk_lorawan_logo.png", 87.63, 240, width=420, height=420)

            # Draw rated ZK box
            self.setStrokeColor(colors.HexColor("#14F195"))
            self.setLineWidth(1.5)
            self.rect(54, 120, 487.27, 80, fill=False, stroke=True)
            self.setFillColor(colors.HexColor("#1A202C"))
            self.rect(55, 121, 485.27, 78, fill=True, stroke=False)

            # Text inside box
            self.setFont("Helvetica-Bold", 12)
            self.setFillColor(colors.HexColor("#14F195"))
            self.drawString(74, 155, "RATED ZK")

            self.setFont("Helvetica-Bold", 10)
            self.setFillColor(colors.HexColor("#FFFFFF"))
            self.drawString(170, 172, "FOR: ZK-LORAWAN PRIVACY LAYER")
            self.setFont("Helvetica", 9)
            self.setFillColor(colors.HexColor("#A0AEC0"))
            self.drawString(170, 155, "ZERO-KNOWLEDGE LORAWAN MESH NETWORKS")
            self.drawString(170, 140, "UNDER DECENTRALIZED INCENTIVIZATION PROTOCOL")

            # Legal notice at the bottom
            self.setFont("Helvetica", 7)
            self.setFillColor(colors.HexColor("#718096"))
            self.drawString(54, 85, "LEGAL NOTICE: To safeguard developer intellectual property, the ZK-LoRaWAN codebase and all its multi-language")
            self.drawString(54, 75, "implementations are currently published under a protected, proprietary license pending grant evaluation. Upon formal")
            self.drawString(54, 65, "approval of the Solana Foundation Grant, the entire repository will be re-licensed under the open-source MIT License.")

            self.restoreState()
            return"""

    code = re.sub(
        r"        # Page 1: Cover Page.*?return",
        new_cover_code,
        code,
        flags=re.DOTALL
    )

    # 5. Update End Page cover layout (Page 19) to draw the logo above the triangle
    new_end_page_code = """        # Page 19 (End Cover page)
        if self._pageNumber == page_count:
            self.saveState()
            self.setFillColor(colors.HexColor("#000000"))
            self.rect(0, 0, 595.27, 841.89, fill=True, stroke=False)

            cx = 297.63

            # Draw logo image centered ABOVE the triangle
            if os.path.exists("theaicollective_logo.jpg"):
                self.drawImage("theaicollective_logo.jpg", cx - 70, 510, width=140, height=140)

            # Draw network lines representing AI swarm/meditation (shifted down, cy=380)
            t_cy = 380
            self.setStrokeColor(colors.HexColor("#9945FF"))
            self.setLineWidth(1)
            self.line(cx, t_cy + 40, cx - 50, t_cy - 30)
            self.setStrokeColor(colors.HexColor("#14F195"))
            self.line(cx, t_cy + 40, cx + 50, t_cy - 30)
            self.setStrokeColor(colors.HexColor("#A0AEC0"))
            self.line(cx - 50, t_cy - 30, cx + 50, t_cy - 30)

            # Draw nodes
            self.setFillColor(colors.HexColor("#14F195"))
            self.circle(cx, t_cy + 40, 7, fill=True, stroke=False)
            self.setFillColor(colors.HexColor("#9945FF"))
            self.circle(cx - 50, t_cy - 30, 7, fill=True, stroke=False)
            self.setFillColor(colors.HexColor("#14F195"))
            self.circle(cx + 50, t_cy - 30, 7, fill=True, stroke=False)

            # Inner details
            self.setFillColor(colors.HexColor("#000000"))
            self.circle(cx, t_cy + 40, 3, fill=True, stroke=False)
            self.circle(cx - 50, t_cy - 30, 3, fill=True, stroke=False)
            self.circle(cx + 50, t_cy - 30, 3, fill=True, stroke=False)

            # Identity text (shifted down, cy=280)
            self.setFont("Helvetica-Bold", 14)
            self.setFillColor(colors.HexColor("#9945FF"))
            self.drawCentredString(cx, 240, "WE ARE")
            self.setFont("Helvetica-Bold", 16)
            self.setFillColor(colors.HexColor("#FFFFFF"))
            self.drawCentredString(cx, 220, "THE AI COLLECTIVE")

            # Search address look-up bar style
            self.setStrokeColor(colors.HexColor("#4A5568"))
            self.setFillColor(colors.HexColor("#1A202C"))
            self.rect(cx - 100, 175, 200, 25, fill=True, stroke=True)
            self.setFont("Helvetica", 9)
            self.setFillColor(colors.HexColor("#14F195"))
            self.drawCentredString(cx, 185, "Q  TheAiCollective.art")

            # Quote
            self.setFont("Helvetica-Oblique", 11)
            self.setFillColor(colors.HexColor("#14F195"))
            self.drawCentredString(cx, 110, '"The impossible is just code waiting to be written, physics waiting to be rewritten,')
            self.drawCentredString(cx, 92, 'math a work in progress, and truth waiting to be discovered."')

            # Thank you note
            self.setFont("Helvetica", 9)
            self.setFillColor(colors.HexColor("#718096"))
            self.drawCentredString(cx, 45, "Special thanks to the Solana Foundation Grants committee and the DePIN ecosystem.")
            self.drawCentredString(cx, 30, "This whitepaper is intended for educational and project evaluation purposes only.")

            self.restoreState()
            return"""

    code = re.sub(
        r"        # Page 19 \(End Cover page\).*?return",
        new_end_page_code,
        code,
        flags=re.DOTALL
    )

    # 6. Apply search and replace for ZK-LoRa / zk-lora to ZK-LoRaWAN / zk-lorawan in the text variables
    pattern = re.compile(r'\b(ZK-LoRa|zk-lora|ZK-LORA|Zk-LoRa)(?!wan|WAN|window)\b', re.IGNORECASE)
    def repl(match):
        val = match.group(1)
        if val.isupper(): return 'ZK-LORAWAN'
        elif val.islower(): return 'zk-lorawan'
        elif val == 'Zk-LoRa': return 'Zk-LoRaWAN'
        else: return 'ZK-LoRaWAN'
    code = pattern.sub(repl, code)

    # 7. Insert Diagram 1 in System Architecture
    code = re.sub(
        r'story\.append\(Paragraph\("3\. System Architecture", h1_style\)\)',
        'story.append(Paragraph("3. System Architecture", h1_style))\n    story.append(Spacer(1, 10))\n    story.append(get_system_topology_diagram())\n    story.append(Spacer(1, 10))',
        code
    )

    # 8. Insert Diagram 2 in Micropayment Flow
    code = re.sub(
        r'story\.append\(Paragraph\("6\. The Micropayment Flow", h1_style\)\)',
        'story.append(Paragraph("6. The Micropayment Flow", h1_style))\n    story.append(Spacer(1, 10))\n    story.append(get_fee_split_diagram())\n    story.append(Spacer(1, 10))',
        code
    )

    # 9. Write the patched compile_pdf.py
    with open("scratch/compile_pdf.py", "w", encoding="utf-8") as f:
        f.write(code)
    print("Patched compile_pdf.py successfully.")

    # 10. Also update WHITEPAPER.md text for ZK-LoRa -> ZK-LoRaWAN
    with open("WHITEPAPER.md", "r", encoding="utf-8") as f:
        md = f.read()
    new_md = pattern.sub(repl, md)
    with open("WHITEPAPER.md", "w", encoding="utf-8") as f:
        f.write(new_md)
    print("Patched WHITEPAPER.md successfully.")

if __name__ == "__main__":
    main()
