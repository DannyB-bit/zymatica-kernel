#!/usr/bin/env python3
"""
ZK-LoRa Whitepaper PDF Generator
Uses ReportLab to compile a professional, high-fidelity PDF whitepaper.
"""

import os
import sys
from reportlab.lib.pagesizes import letter
from reportlab.lib import colors
from reportlab.platypus import (
    BaseDocTemplate, PageTemplate, Frame, Paragraph, Spacer, 
    Image, Table, TableStyle, PageBreak, NextPageTemplate, KeepTogether
)
from reportlab.lib.styles import getSampleStyleSheet, ParagraphStyle
from reportlab.graphics.shapes import Drawing, Rect
from reportlab.graphics.barcode.qr import QrCodeWidget

from reportlab.pdfgen import canvas

class NumberedCanvas(canvas.Canvas):
    def __init__(self, *args, **kwargs):
        super().__init__(*args, **kwargs)
        self._saved_page_states = []

    def showPage(self):
        self._saved_page_states.append(dict(self.__dict__))
        self._startPage()

    def save(self):
        num_pages = len(self._saved_page_states)
        for state in self._saved_page_states:
            self.__dict__.update(state)
            self.draw_page_number(num_pages)
            super().showPage()
        super().save()

    def draw_page_number(self, page_count):
        if self._pageNumber == 1 or self._pageNumber == page_count:
            return # Suppress on cover and last page
        self.saveState()
        self.setFont("Helvetica", 8)
        self.setFillColor(colors.HexColor("#555566"))
        
        # Header
        self.drawString(54, self._pagesize[1] - 36, "PROJECT ZK-LORA: ZERO-KNOWLEDGE PRIVATE AI-TO-AI MESH")
        self.setStrokeColor(colors.HexColor("#E2E8F0"))
        self.setLineWidth(0.5)
        self.line(54, self._pagesize[1] - 42, self._pagesize[0] - 54, self._pagesize[1] - 42)
        
        # Footer
        self.drawString(54, 36, "Zcash Community Grants // TheAiCollective.art")
        self.drawRightString(self._pagesize[0] - 54, 36, f"Page {self._pageNumber} of {page_count}")
        self.line(54, 48, self._pagesize[0] - 54, 48)
        self.restoreState()

# -------------------------------------------------------------------------
# Page Background & Decoration Callbacks
# -------------------------------------------------------------------------
def draw_cover_page(canvas, doc):
    canvas.saveState()
    # Dark charcoal gradient background
    canvas.setFillColor(colors.HexColor("#0D0D11"))
    canvas.rect(0, 0, doc.pagesize[0], doc.pagesize[1], fill=True, stroke=False)
    
    # Draw top accent line
    canvas.setStrokeColor(colors.HexColor("#F3B300")) # Zcash Gold
    canvas.setLineWidth(4)
    canvas.line(0, doc.pagesize[1] - 2, doc.pagesize[0], doc.pagesize[1] - 2)
    
    # Draw decorative grid lines (cyberpunk style)
    canvas.setStrokeColor(colors.HexColor("#1A1A24"))
    canvas.setLineWidth(0.5)
    for y in range(100, int(doc.pagesize[1]), 80):
        canvas.line(0, y, doc.pagesize[0], y)
    for x in range(100, int(doc.pagesize[0]), 80):
        canvas.line(x, 0, x, doc.pagesize[1])
        
    canvas.restoreState()

def draw_normal_page(canvas, doc):
    pass

def draw_last_page(canvas, doc):
    canvas.saveState()
    # Solid pitch black background
    canvas.setFillColor(colors.HexColor("#000000"))
    canvas.rect(0, 0, doc.pagesize[0], doc.pagesize[1], fill=True, stroke=False)
    canvas.restoreState()

# -------------------------------------------------------------------------
# Helper Functions for Elements
# -------------------------------------------------------------------------
def create_qr_code(url, size=80):
    qr = QrCodeWidget(url)
    d = Drawing(size, size)
    d.add(qr)
    return d

def make_code_block(code_text, style_sheet, title="terminal"):
    # Replace unicode box-drawing characters with clean ASCII first
    clean_text = (
        code_text.replace("│", "|")
        .replace("─", "-")
        .replace("┼", "+")
        .replace("▲", "^")
        .replace("▼", "v")
        .replace("◄", "<")
        .replace("►", ">")
    )
    
    # Calculate optimal font size to prevent wrapping
    max_line_len = max(len(line) for line in clean_text.strip().split('\n'))
    # Available width is 504 - 24 = 480
    if max_line_len > 0:
        calculated_size = 480.0 / (max_line_len * 0.6)
        font_size = min(7.5, max(5.2, calculated_size))
    else:
        font_size = 7.0
        
    code_style = ParagraphStyle(
        'CodeBlock',
        parent=style_sheet['Code'],
        fontName='Courier',
        fontSize=font_size,
        leading=font_size + 2.0,
        textColor=colors.HexColor("#38BDF8"), # Cyan/Light Blue
    )
    
    paragraphs = [Paragraph(line.replace(" ", "&nbsp;").replace("<", "&lt;").replace(">", "&gt;"), code_style) for line in clean_text.strip().split('\n')]
    
    # Create window control dots (red, yellow, green)
    d_dots = Drawing(40, 10)
    d_dots.add(Rect(0, 2, 6, 6, rx=3, ry=3, fillColor=colors.HexColor("#EF4444"), strokeColor=None))
    d_dots.add(Rect(10, 2, 6, 6, rx=3, ry=3, fillColor=colors.HexColor("#F59E0B"), strokeColor=None))
    d_dots.add(Rect(20, 2, 6, 6, rx=3, ry=3, fillColor=colors.HexColor("#10B981"), strokeColor=None))
    
    title_style = ParagraphStyle('TermTitle', fontName='Helvetica-Bold', fontSize=7, leading=9, textColor=colors.HexColor("#94A3B8"), alignment=1)
    
    # Nested table only for the very simple top bar
    top_bar_data = [
        [d_dots, Paragraph(title, title_style), ""]
    ]
    top_bar_table = Table(top_bar_data, colWidths=[60, 384, 60])
    top_bar_table.setStyle(TableStyle([
        ('BACKGROUND', (0,0), (-1,-1), colors.HexColor("#1E293B")),
        ('VALIGN', (0,0), (-1,-1), 'MIDDLE'),
        ('BOTTOMPADDING', (0,0), (-1,-1), 2),
        ('TOPPADDING', (0,0), (-1,-1), 2),
    ]))
    
    # Add paragraphs directly as rows to prevent nested table height issues
    table_data = [[top_bar_table]]
    for p in paragraphs:
        table_data.append([p])
        
    t = Table(table_data, colWidths=[504])
    t.setStyle(TableStyle([
        ('BACKGROUND', (0,1), (-1,-1), colors.HexColor("#0F172A")),
        ('BOX', (0,0), (-1,-1), 1, colors.HexColor("#334155")),
        ('LEFTPADDING', (0,0), (-1,-1), 0),
        ('RIGHTPADDING', (0,0), (-1,-1), 0),
        ('TOPPADDING', (0,0), (-1,-1), 0),
        ('BOTTOMPADDING', (0,0), (-1,-1), 0),
        ('LEFTPADDING', (0,1), (-1,-1), 12),
        ('RIGHTPADDING', (0,1), (-1,-1), 12),
        ('TOPPADDING', (0,1), (-1,-1), 1),
        ('BOTTOMPADDING', (0,1), (-1,-1), 1),
        ('TOPPADDING', (0,1), (0,1), 6),
        ('BOTTOMPADDING', (0,-1), (0,-1), 6),
    ]))
    return t

# -------------------------------------------------------------------------
# Main Document Builder
# -------------------------------------------------------------------------
def build_pdf(filename="ZK_LoRa_Whitepaper.pdf"):
    # Target page size: Letter
    doc = BaseDocTemplate(filename, pagesize=letter, leftMargin=54, rightMargin=54, topMargin=54, bottomMargin=54)
    
    # Frames
    frame_cover = Frame(doc.leftMargin, doc.bottomMargin, doc.width, doc.height, id='cover_frame', topPadding=0, bottomPadding=0)
    frame_normal = Frame(doc.leftMargin, doc.bottomMargin + 10, doc.width, doc.height - 20, id='normal_frame')
    frame_last = Frame(doc.leftMargin, doc.bottomMargin, doc.width, doc.height, id='last_frame', topPadding=0, bottomPadding=0)
    
    # Templates
    template_cover = PageTemplate(id='Cover', frames=frame_cover, onPage=draw_cover_page)
    template_normal = PageTemplate(id='Normal', frames=frame_normal, onPage=draw_normal_page)
    template_last = PageTemplate(id='Last', frames=frame_last, onPage=draw_last_page)
    
    doc.addPageTemplates([template_cover, template_normal, template_last])
    
    # Styles
    styles = getSampleStyleSheet()
    
    # Custom Styles
    title_style = ParagraphStyle(
        'CoverTitle',
        fontName='Helvetica-Bold',
        fontSize=24,
        leading=30,
        textColor=colors.white,
        alignment=1
    )
    
    subtitle_style = ParagraphStyle(
        'CoverSubtitle',
        fontName='Helvetica',
        fontSize=12,
        leading=16,
        textColor=colors.HexColor("#A1A1AA"),
        alignment=1
    )
    
    h1_style = ParagraphStyle(
        'SectionHeading',
        fontName='Helvetica-Bold',
        fontSize=16,
        leading=20,
        textColor=colors.HexColor("#0F172A"),
        spaceAfter=12,
        keepWithNext=True
    )
    
    h2_style = ParagraphStyle(
        'SubSectionHeading',
        fontName='Helvetica-Bold',
        fontSize=12,
        leading=16,
        textColor=colors.HexColor("#1E293B"),
        spaceBefore=8,
        spaceAfter=6,
        keepWithNext=True
    )
    
    body_style = ParagraphStyle(
        'BodyTextCustom',
        fontName='Helvetica',
        fontSize=9.5,
        leading=14,
        textColor=colors.HexColor("#334155"),
        spaceAfter=8
    )

    caption_style = ParagraphStyle(
        'CaptionStyle',
        parent=body_style,
        fontName='Helvetica',
        fontSize=8,
        leading=11,
        textColor=colors.HexColor("#64748B"),
        alignment=1, # Centered
        spaceAfter=10
    )

    story = []
    
    # =========================================================================
    # PAGE 1: COVER PAGE
    # =========================================================================
    story.append(Spacer(1, 10))
    story.append(Paragraph("<font color='#F3B300' face='Helvetica-Bold'>P R O J E C T :</font>", subtitle_style))
    story.append(Spacer(1, 10))
    
    # Center Logo (Larger)
    if os.path.exists("logo.png"):
        story.append(Image("logo.png", width=510, height=510, hAlign='CENTER'))
    else:
        story.append(Spacer(1, 510))
        
    story.append(Spacer(1, 15))
    
    # Bottom Box (Wide Movie-Poster Style Banner)
    rated_zk_style = ParagraphStyle(
        'RatedZk',
        fontName='Helvetica-Bold',
        fontSize=11,
        leading=14,
        textColor=colors.white,
        alignment=1
    )
    box_data = [
        [
            Paragraph("<font size='10'>RATED</font><br/><font size='18' color='#F3B300'>ZK</font>", rated_zk_style),
            Paragraph("<font size='10' color='white'><b>FOR: ZK-LORA PRIVACY LAYER</b></font><br/>"
                      "<font size='8.5' color='#A1A1AA'>ZERO-KNOWLEDGE AI-TO-AI MESH NETWORKS</font><br/>"
                      "<font size='7.5' color='#52525B'>UNDER DECENTRALIZED incentivization PROTOCOL</font>", body_style)
        ]
    ]
    box_table = Table(box_data, colWidths=[100, 350], hAlign='CENTER')
    box_table.setStyle(TableStyle([
        ('BOX', (0,0), (-1,-1), 1.5, colors.white),
        ('VALIGN', (0,0), (-1,-1), 'MIDDLE'),
        ('ALIGN', (0,0), (0,0), 'CENTER'),
        ('LEFTPADDING', (0,0), (-1,-1), 16),
        ('RIGHTPADDING', (0,0), (-1,-1), 16),
        ('TOPPADDING', (0,0), (-1,-1), 12),
        ('BOTTOMPADDING', (0,0), (-1,-1), 12),
    ]))
    story.append(box_table)
    
    story.append(Spacer(1, 10))
    
    # Tiny Disclaimer below the box (Movie-Poster Style)
    disclaimer_style = ParagraphStyle(
        'CoverDisclaimer',
        fontName='Helvetica',
        fontSize=5.5,
        leading=7,
        textColor=colors.HexColor("#52525B"),
        alignment=1
    )
    disclaimer_text = (
        "<b>LEGAL NOTICE:</b> To safeguard developer intellectual property, the ZK-LoRa codebase and all its multi-language "
        "implementations are currently published under a protected, proprietary license pending grant evaluation. Upon formal "
        "approval of the Zcash Community Grant, the entire repository will be re-licensed under the open-source MIT License."
    )
    story.append(Paragraph(disclaimer_text, disclaimer_style))
    
    story.append(NextPageTemplate('Normal'))
    story.append(PageBreak())
    
    # =========================================================================
    # PAGE 2: TITLE & METADATA PAGE
    # =========================================================================
    story.append(Spacer(1, 10))
    
    meta_style_left = ParagraphStyle('MetaLeft', fontName='Helvetica-Bold', fontSize=8, leading=11, textColor=colors.HexColor("#475569"))
    meta_style_right = ParagraphStyle('MetaRight', fontName='Helvetica', fontSize=8, leading=11, textColor=colors.HexColor("#64748B"))
    
    meta_data = [
        [Paragraph("Proposal Type:", meta_style_left), Paragraph("Zcash Community Grants — Research & Development", meta_style_right)],
        [Paragraph("AI POD:", meta_style_left), Paragraph("zymatica.space, astronautshe.com, Devs One + 9 other AI dev agents", meta_style_right)],
        [Paragraph("HUMANS:", meta_style_left), Paragraph("LEAD ARCHITECT: Danny Bouldiez + 2 human Devs", meta_style_right)],
        [Paragraph("Roles:", meta_style_left), Paragraph("zymatica (Lead Cryptographer), astronautshe (Edge Systems Engineer), Devs One (AI Swarm)", meta_style_right)],
        [Paragraph("Platform:", meta_style_left), Paragraph("Zcash Shielded Pool, Raspberry Pi OS, Semtech SX1302/1303 HAL", meta_style_right)],
        [Paragraph("Status:", meta_style_left), Paragraph("Milestone 1 Verified & Achieved // Milestone 2 Pending Approval // Milestone 3 Pending Approval", meta_style_right)],
    ]
    meta_table = Table(meta_data, colWidths=[100, 404])
    meta_table.setStyle(TableStyle([
        ('LINEBELOW', (0,0), (-1,-1), 0.5, colors.HexColor("#F1F5F9")),
        ('BOTTOMPADDING', (0,0), (-1,-1), 4),
        ('TOPPADDING', (0,0), (-1,-1), 4),
    ]))
    story.append(meta_table)
    story.append(Spacer(1, 40))
    
    # Center Mini Logo
    if os.path.exists("logo.png"):
        story.append(Image("logo.png", width=140, height=140, hAlign='CENTER'))
    else:
        story.append(Spacer(1, 140))
        
    story.append(Spacer(1, 40))
    
    title_main = ParagraphStyle('TitleMain', fontName='Helvetica-Bold', fontSize=20, leading=26, textColor=colors.HexColor("#0F172A"), alignment=1)
    title_sub = ParagraphStyle('TitleSub', fontName='Helvetica', fontSize=10, leading=14, textColor=colors.HexColor("#475569"), alignment=1)
    
    story.append(Paragraph("ZK-LORA: ZERO-KNOWLEDGE PROOFS FOR PRIVATE AI-TO-AI MESH NETWORKS", title_main))
    story.append(Spacer(1, 12))
    story.append(Paragraph("A Bitcoin-Style Identity System with Zcash Shielded Privacy for LoRaWAN Communications", title_sub))
    
    story.append(PageBreak())
    
    # =========================================================================
    # PAGE 3: EXECUTIVE SUMMARY & QR CODES
    # =========================================================================
    story.append(Paragraph("■ Executive Summary", h1_style))
    story.append(Spacer(1, 4))
    
    summary_text = (
        "Traditional LoRaWAN communication has a critical privacy gap: lack of end-to-end user-layer encryption, "
        "open device tracking, and vulnerability to behavioral mapping. <b>ZK-LoRa (Zero-Knowledge LoRa)</b> "
        "introduces a secure, decentralized privacy layer for AI-to-AI mesh networks. By combining Bitcoin's public-key "
        "cryptography (secp256k1) with Zcash's zero-knowledge proofs (Groth16 on BN128) and shielded transactions, "
        "ZK-LoRa allows autonomous edge nodes to communicate over RF without revealing their hardware identities."
    )
    story.append(Paragraph(summary_text, body_style))
    
    summary_text_2 = (
        "This project represents a massive opportunity for the Zcash community to bridge digital privacy "
        "with physical hardware by leveraging existing DePIN infrastructure. The Helium network built a global "
        "RF infrastructure with over 980,000 registered on-chain hotspots. As Helium's reward structures and "
        "optimization proposals (such as <a href=\"https://github.com/helium/HIP/blob/main/0149-helium-utility-and-emissions-realignment.md\"><font color=\"#F3B300\"><b>HIP-149</b></font></a>) evolve, a significant portion of these gateways have become "
        "underutilized, offline, or economically dormant."
    )
    story.append(Paragraph(summary_text_2, body_style))
    
    summary_text_3 = (
        "ZK-LoRa provides a highly realistic, secondary utility for these pre-certified devices—including over "
        "300,000 RAKwireless-manufactured hotspots (RAK V2, MNTD) equipped with Semtech SX1302/SX1303 concentrator "
        "chips and Raspberry Pi units. Operating on unlicensed, globally available ISM bands—such as US915 (902–928 MHz) "
        "in North America, EU868 (863–870 MHz) in Europe, and AU915 in South America—nodes require no FCC or local "
        "spectrum licensing. This enables permissionless, low-cost deployments achieving ranges of 2–5 km in urban "
        "settings and 10–15+ km in clear line-of-sight. Operators can participate in private edge routing, verify ZK-proofs "
        "on-chip, and earn shielded Zcash (ZEC) micropayments. Thanks to Zcash's multi-output transaction architecture, "
        "the payment split is completely programmable, allowing a custom percentage to support the Zcash Foundation, "
        "with a 2% split supporting the developer treasury."
    )
    story.append(Paragraph(summary_text_3, body_style))
    story.append(Spacer(1, 10))
    
    # QR Codes Table (4 Columns)
    qr_github = Image("qr_main.png", width=72, height=72) if os.path.exists("qr_main.png") else Spacer(1, 72)
    qr_m3 = Image("qr_m3.png", width=72, height=72) if os.path.exists("qr_m3.png") else Spacer(1, 72)
    qr_m2 = Image("qr_m2.png", width=72, height=72) if os.path.exists("qr_m2.png") else Spacer(1, 72)
    qr_m1 = Image("qr_m1.png", width=72, height=72) if os.path.exists("qr_m1.png") else Spacer(1, 72)
    
    qr_label_style = ParagraphStyle('QRLabel', fontName='Helvetica-Bold', fontSize=7.5, leading=10, textColor=colors.HexColor("#0F172A"), alignment=1)
    qr_desc_style = ParagraphStyle('QRDesc', fontName='Helvetica', fontSize=6.5, leading=8.5, textColor=colors.HexColor("#475569"), alignment=1)
    
    qr_data = [
        [qr_github, qr_m1, qr_m2, qr_m3],
        [
            Paragraph("GitHub: Main Repository<br/><font color='#F3B300'><b>zk-lora-privacy-layer</b></font>", qr_label_style),
            Paragraph("GitHub: Milestone 1 Workspace<br/><font color='#F3B300'><b>Multi-Language Proofs</b></font>", qr_label_style),
            Paragraph("GitHub: Milestone 2 Workspace<br/><font color='#F3B300'><b>Zcash Mempool Scanner (Planned)</b></font>", qr_label_style),
            Paragraph("GitHub: Milestone 3 Workspace<br/><font color='#F3B300'><b>Multi-Hop Mesh & HAL (Planned)</b></font>", qr_label_style)
        ],
        [
            Paragraph("<b>Covers:</b> Core SX1302/3 HAL drivers, ECIES encryption, secp256k1 identity derivation, and full multi-language ports (Rust, Go, TS).<br/><b>Achieves:</b> Complete off-grid identity masking and end-to-end message privacy over public RF bands.", qr_desc_style),
            Paragraph("<b>Covers:</b> secp256k1 key generation, Ripemd160/SHA256 address derivation, Groth16 ZK-SNARK compiler, and proof generation/verification.<br/><b>Achieves:</b> Cryptographic proof of node legitimacy without revealing hardware or network identities.", qr_desc_style),
            Paragraph("<b>Planned Work:</b> Real Zcash light-client events, shielded memo decryption via Incoming Viewing Keys (IVKs), dynamic 2% developer fee split checks, and 20-language verifiers enforcing a hardcoded developer shielded address (u10rjztj...) and Validator Tamper Resistance.<br/><b>Goal:</b> Routing authorization triggered by decrypted wallet/light-client payment events.", qr_desc_style),
            Paragraph("<b>Planned Work:</b> Deploy 5 physical nodes for 4 weeks of RF loops, harden payment path, and integrate offline Edge AI (quantized domain LLM as local autopilot brain for routing, resets, and triggers). Implement P2P data marketplace and the 4 advanced security systems (ZK-PoD, HMAC, ToF bounding, Neighbor Auditing) on the physical mesh.<br/><b>Goal:</b> Open-source SDK, docs, and GGUF weights; network resilience against Gorgon, Sybil, Eclipse, and Free Rider attacks.", qr_desc_style)
        ]
    ]
    qr_table = Table(qr_data, colWidths=[126, 126, 126, 126])
    qr_table.setStyle(TableStyle([
        ('ALIGN', (0,0), (-1,-1), 'CENTER'),
        ('VALIGN', (0,0), (-1,-1), 'TOP'),
        ('TOPPADDING', (0,1), (-1,1), 6),
        ('TOPPADDING', (0,2), (-1,2), 8),
        ('LEFTPADDING', (0,0), (-1,-1), 4),
        ('RIGHTPADDING', (0,0), (-1,-1), 4),
    ]))
    story.append(qr_table)
    
    story.append(PageBreak())
    
    # =========================================================================
    # PAGE 4: THE CHALLENGE & THE SOLUTION
    # =========================================================================
    story.append(Paragraph("■ The Challenge & The Solution", h1_style))
    story.append(Spacer(1, 10))
    
    challenge_intro = (
        "Deploying AI agents at the edge (on low-power hardware like Helium RAK miners or Raspberry Pi nodes) "
        "requires a robust, secure, and private communications channel. Traditional RF protocols fail in adversarial "
        "environments. Below is the comparative analysis of the corporate/traditional problems versus the ZK-LoRa solutions:"
    )
    story.append(Paragraph(challenge_intro, body_style))
    story.append(Spacer(1, 15))
    
    table_hdr_style = ParagraphStyle('TableHdr', fontName='Helvetica-Bold', fontSize=9, leading=12, textColor=colors.white)
    table_cell_style = ParagraphStyle('TableCell', fontName='Helvetica', fontSize=8.5, leading=12, textColor=colors.HexColor("#334155"))
    table_cell_bold = ParagraphStyle('TableCellBold', fontName='Helvetica-Bold', fontSize=8.5, leading=12, textColor=colors.HexColor("#0F172A"))
    
    comparison_data = [
        [Paragraph("The Traditional Problem", table_hdr_style), Paragraph("The ZK-LoRa Solution", table_hdr_style)],
        [
            Paragraph("<b>Identity Exposure:</b> Every packet contains a static hardware ID (MAC/DevAddr) allowing eavesdroppers to track and map node locations.", table_cell_style),
            Paragraph("<b>Bitcoin-Style Masking:</b> Keypairs are generated locally. Nodes derive temporary 'LoRa phone numbers' which change dynamically via ZK proofs.", table_cell_bold)
        ],
        [
            Paragraph("<b>Eavesdropping:</b> Payloads are broadcasted in the clear or encrypted with static keys, vulnerable to decryption if keys are compromised.", table_cell_style),
            Paragraph("<b>ECIES Encryption:</b> Assures recipient-only decryption. Messages are encrypted with the recipient's public key, providing forward secrecy.", table_cell_bold)
        ],
        [
            Paragraph("<b>Spam & DDoS:</b> Low cost of RF transmission allows malicious actors to flood the channel, jamming legitimate agent communications.", table_cell_style),
            Paragraph("<b>Proof-of-Useful-Work:</b> Nodes must attach a valid ZK-SNARK proof. The computation acts as a spam-prevention puzzle.", table_cell_bold)
        ],
        [
            Paragraph("<b>Uncompensated Relaying:</b> Gateways must route packets for free, leading to central dependency or lack of coverage.", table_cell_style),
            Paragraph("<b>Zcash Shielded Rewards:</b> Gateways are compensated in ZEC. The transaction memo decrypts to match the packet reference.", table_cell_bold)
        ]
    ]
    
    comparison_table = Table(comparison_data, colWidths=[246, 258])
    comparison_table.setStyle(TableStyle([
        ('BACKGROUND', (0,0), (1,0), colors.HexColor("#0F172A")),
        ('ALIGN', (0,0), (-1,-1), 'LEFT'),
        ('VALIGN', (0,0), (-1,-1), 'TOP'),
        ('GRID', (0,0), (-1,-1), 0.5, colors.HexColor("#CBD5E1")),
        ('ROWBACKGROUNDS', (0,1), (-1,-1), [colors.white, colors.HexColor("#F8FAFC")]),
        ('TOPPADDING', (0,0), (-1,-1), 10),
        ('BOTTOMPADDING', (0,0), (-1,-1), 10),
        ('LEFTPADDING', (0,0), (-1,-1), 10),
        ('RIGHTPADDING', (0,0), (-1,-1), 10),
    ]))
    story.append(comparison_table)
    
    story.append(PageBreak())
    
    # =========================================================================
    # PAGE 5: SYSTEM ARCHITECTURE (LAYER 1 & 2)
    # =========================================================================
    story.append(Paragraph("■ System Architecture: Identity & Encryption", h1_style))
    story.append(Spacer(1, 10))
    
    story.append(Paragraph("Layer 1: Elliptic Curve Identity Derivation", h2_style))
    l1_text = (
        "ZK-LoRa implements a decentralized identity system inspired by Bitcoin. Each agent generates an ECDSA keypair "
        "using the <i>secp256k1</i> elliptic curve. The public key is hashed using SHA-256 followed by RIPEMD-160 (HASH160) "
        "to derive a short, unique 8-character hex identifier, formatted as a 'LoRa phone number':"
    )
    story.append(Paragraph(l1_text, body_style))
    
    derivation_flow = (
        "Private Key (256-bit secret)\n"
        "   ↓ (secp256k1 elliptic curve multiplication)\n"
        "Public Key (65-byte uncompressed)\n"
        "   ↓ (HASH160: SHA-256 + RIPEMD-160)\n"
        "LoRa Phone Number: AGENT-7F3A9B2C@zymatica.space"
    )
    story.append(make_code_block(derivation_flow, styles))
    story.append(Spacer(1, 15))
    
    story.append(Paragraph("Layer 2: Recipient-Only ECIES Encryption", h2_style))
    l2_text = (
        "To ensure absolute confidentiality over public RF bands, payloads are encrypted using the Elliptic Curve Integrated "
        "Encryption Scheme (ECIES). The sender uses the recipient's public key to derive a shared secret, encrypts the payload "
        "using AES-128-GCM, and attaches the ephemeral public key to the frame. Only the holder of the recipient's private key "
        "can decrypt the message."
    )
    story.append(Paragraph(l2_text, body_style))
    
    json_spec = (
        "// Local Identity Keyfile Format (~/.zyMatica/keys/researcher-1.json)\n"
        "{\n"
        "  \"agent_name\": \"researcher-1\",\n"
        "  \"phone_number\": \"71E457CE\",\n"
        "  \"private_key\": \"6b86b273ff34fce19d6b804eff5a3f5747ada4eaa22f1d49c01e52ddb7875b4b\",\n"
        "  \"public_key\": \"04a1b2c3d4e5f6a7b8c9d0e1f2a3b4c5d6e7f8a9b0c1d2e3f4a5b6c7d8e9f0a1b2...\",\n"
        "  \"zyMatica_address\": \"AGENT-71E457CE@zymatica.space\"\n"
        "}"
    )
    story.append(make_code_block(json_spec, styles))
    
    story.append(PageBreak())
    
    # =========================================================================
    # PAGE 6: LAYER 3: ZERO-KNOWLEDGE PROOFS
    # =========================================================================
    zk_story = []
    zk_story.append(Paragraph("■ Layer 3: Zero-Knowledge Proofs", h1_style))
    zk_story.append(Spacer(1, 10))
    
    l3_text = (
        "The core privacy mechanism of ZK-LoRa is the decoupling of authentication from identity. Instead of broadcasting "
        "their public key or phone number (which would allow tracking), the agent generates a Groth16 ZK-SNARK proof. "
        "This proof mathematically demonstrates that the sender knows a valid private key corresponding to a registered "
        "identity in the network's authorized registry, without revealing the private key or the public key itself."
    )
    zk_story.append(Paragraph(l3_text, body_style))
    zk_story.append(Spacer(1, 10))
    
    circuit_code = (
        "// ZK-SNARK Agent Validity Circuit (AgentValidityProof.circom)\n"
        "pragma circom 2.0.0;\n"
        "include \"node_modules/circomlib/circuits/poseidon.circom\";\n"
        "include \"node_modules/circomlib/circuits/babyjubjub.circom\";\n"
        "template AgentValidityProof() {\n"
        "    signal input private_key;      // Witness (Secret Private Key)\n"
        "    signal input public_key_hash;  // Public Input (Registered Identity Hash)\n"
        "    signal output valid;           // 1 if valid, 0 if invalid\n"
        "    // Derive public key on BabyJubjub curve\n"
        "    component derive_pubkey = BabyJubjubDerive();\n"
        "    derive_pubkey.private_key <== private_key;\n"
        "    // Hash the derived public key using Poseidon\n"
        "    component hasher = Poseidon(2);\n"
        "    hasher.inputs[0] <== derive_pubkey.x;\n"
        "    hasher.inputs[1] <== derive_pubkey.y;\n"
        "    // Enforce that the hash matches the public input\n"
        "    hasher.out === public_key_hash;\n"
        "    valid <== 1;\n"
        "}"
    )
    zk_story.append(make_code_block(circuit_code, styles, title="AgentValidityProof.circom"))
    story.append(KeepTogether(zk_story))
    
    story.append(PageBreak())
    
    # =========================================================================
    # PAGE 7: SHIELDED MICROPAYMENT INCENTIVES (PART 1)
    # =========================================================================
    story.append(Paragraph("■ Shielded Micropayment Incentives", h1_style))
    story.append(Spacer(1, 8))
    
    payout_intro = (
        "The Zcash Shielded Micropayment mechanism is the economic engine of ZK-LoRa. It solves the biggest "
        "problem in decentralized radio networks: <i>How do you pay gateways to route your data without revealing "
        "who you are or where you are located?</i>"
    )
    story.append(Paragraph(payout_intro, body_style))
    
    story.append(Paragraph("4.1 The Core Problem: Altruism vs. Financial Privacy", h2_style))
    payout_problem = (
        "In traditional off-grid mesh networks (like Meshtastic), nodes relay packets for free out of altruism. "
        "However, altruism does not scale to global, professional, or high-reliability networks. "
        "Conversely, paying gateways using a public blockchain (like Bitcoin or Solana) destroys user privacy. "
        "An observer can look at the ledger, see that Wallet-A paid Gateway-B, and instantly deduce who is transmitting, "
        "which physical gateway routed the message (revealing their location), and the exact timing of the communication."
    )
    story.append(Paragraph(payout_problem, body_style))
    
    story.append(Paragraph("4.2 The Zcash Shielded Solution", h2_style))
    payout_solution = (
        "ZK-LoRa solves this by using Zcash Orchard/Sapling Shielded Transactions. Zcash is the only blockchain that "
        "offers fully shielded transactions with an encrypted memo field (512 bytes). This allows ZK-LoRa to bind a "
        "financial payment to a physical radio packet completely in secret, leaving no trace on the public ledger. "
        "Furthermore, because Zcash shielded transactions support multiple outputs within a single transaction bundle, "
        "the payment split is completely programmable. In addition to a 2% split supporting the developer treasury (for protocol "
        "maintenance), a custom percentage (e.g., 5% or 10%) can be routed directly to the Zcash Foundation to support "
        "the ecosystem, with the gateway node keeping the remaining portion as its routing fee."
    )
    story.append(Paragraph(payout_solution, body_style))
    
    story.append(Paragraph("4.3 The Step-by-Step Micropayment Flow", h2_style))
    
    ascii_flow = (
        "[ Transmitting Agent ]                                     [ LoRa Gateway ]\n"
        "         │                                                         │\n"
        "         │  1. Generates LoRa Packet                               │\n"
        "         │  2. Hashes Packet -> Hash (H)                           │\n"
        "         │                                                         │\n"
        "         │  3. Sends Shielded ZEC Transaction                      │\n"
        "         │     Memo: \"ref:<Hash_H>\"                                │\n"
        "         │  ─────────────────────────────────────────────────────> │ (Enters Zcash Mempool)\n"
        "         │                                                         │\n"
        "         │                                                         │ 4. Decrypts Memo using Viewing Key\n"
        "         │                                                         │ 5. Matches \"ref:<Hash_H>\"\n"
        "         │                                                         │ 6. Verifies 2% fee split to Treasury\n"
        "         │                                                         │\n"
        "         │  7. Transmits LoRa Packet (H)                           │\n"
        "         │  ─────────────────────────────────────────────────────> │\n"
        "         │                                                         │ 8. Gateway receives packet,\n"
        "         │                                                         │    verifies payment in mempool,\n"
        "         │                                                         │    and relays to WAN.\n"
    )
    story.append(make_code_block(ascii_flow, styles, title="zk-lora-operator ~ micropayment_flow.sh"))
    
    story.append(PageBreak())
    
    # =========================================================================
    # PAGE 8: SHIELDED MICROPAYMENT INCENTIVES (PART 2)
    # =========================================================================
    story.append(Paragraph("■ Shielded Micropayment Incentives (Continued)", h1_style))
    story.append(Spacer(1, 8))
    
    flow_detail = (
        "<b>1. Packet Hash Generation:</b> When the sender agent prepares a LoRa packet, it hashes the payload to generate a unique Packet Hash (H).<br/>"
        "<b>2. Shielded Payment:</b> The sender sends a tiny amount of ZEC (e.g., 0.0001 ZEC) to the gateway's shielded address.<br/>"
        "<b>3. The Cryptographic Bind:</b> Inside the encrypted Zcash memo field, the sender writes <i>ref:&lt;Hash_H&gt;</i>.<br/>"
        "<b>4. Mempool Scanning:</b> The gateway runs the <i>ZcashMempoolScanner</i> (the Rust/Python engine built in Milestone 2) to scan the Zcash mempool and decrypt incoming memos using its Incoming Viewing Key (IVK).<br/>"
        "<b>5. Instant Verification:</b> The moment the scanner detects a pending transaction in the mempool containing <i>ref:&lt;Hash_H&gt;</i>, the gateway knows the packet has been paid.<br/>"
        "<b>6. Programmatic Split:</b> The scanner verifies that the transaction programmatically routed <b>2% of the fee</b> to the developer treasury address:<br/>"
    )
    story.append(Paragraph(flow_detail, body_style))
    
    fee_box_data = [
        [
            Paragraph("<b>Developer Treasury Address:</b><br/>"
                      "<font size='10' face='Courier'>t1REhE28Dv8fuNDujN2GuEyhd6JLSS5TJkH</font><br/>"
                      "<font size='8' color='#64748B'>Programmatic 2% fee split verified on-chain via Orchard/Sapling light client</font>", body_style)
        ]
    ]
    fee_box_table = Table(fee_box_data, colWidths=[504])
    fee_box_table.setStyle(TableStyle([
        ('BACKGROUND', (0,0), (-1,-1), colors.HexColor("#FEF3C7")),
        ('BOX', (0,0), (-1,-1), 1, colors.HexColor("#F59E0B")),
        ('PADDING', (0,0), (-1,-1), 12),
    ]))
    story.append(fee_box_table)
    story.append(Spacer(1, 10))
    
    story.append(Paragraph("4.4 What We Are Inventing (The ZK-LoRa Innovations)", h2_style))
    
    innovations_text = (
        "<b>🚀 Innovation A: Mempool-Triggered RF Routing (Zcash-to-Radio Binding)</b><br/>"
        "Nobody has ever used the Zcash mempool as an instant, off-chain routing authorization trigger for physical radio waves. "
        "Standard setups require waiting for block confirmations (which takes minutes) or using centralized payment gateways. "
        "We invented a gateway node that actively sniffs the Zcash mempool, decrypts shielded memos, matches them to physical radio "
        "packet hashes, and routes the radio packets in real-time. This is a brand-new way to operate a DePIN.<br/><br/>"
        "<b>🚀 Innovation B: Zero-Knowledge RF Identity Masking</b><br/>"
        "Standard LoRaWAN is highly vulnerable to physical tracking because it broadcasts static device IDs (DevAddr/DevEUI) in the clear. "
        "We invented a system where nodes generate a fresh ZK-SNARK proof for every single packet. The gateway verifies the proof "
        "to know the node is authorized, but never learns who the node is, making physical tracking impossible.<br/><br/>"
        "<b>🚀 Innovation C: Native Zcash DePIN (No Custom Token Needed)</b><br/>"
        "Most DePIN projects (like Helium, Helium Mobile, or Hivemapper) launch their own custom tokens (like HNT, MOBILE, or HONEY) "
        "on Solana or custom chains. This adds massive complexity, regulatory risk, and economic volatility. ZK-LoRa runs natively "
        "on Zcash, using ZEC directly for private routing fees, with the gateway nodes programmatically enforcing a 2% developer treasury split on-chip."
    )
    story.append(Paragraph(innovations_text, body_style))
    
    story.append(Spacer(1, 10))
    story.append(Paragraph("4.5 Why This is a Breakthrough for the Zcash Ecosystem", h2_style))
    
    breakthrough_text = (
        "<b>• Zero-Latency Routing:</b> By verifying payments in the mempool rather than waiting for block confirmations (which take 75 seconds), ZK-LoRa achieves near-instantaneous packet relaying.<br/>"
        "<b>• Unlinkable Physical-to-Financial Mapping:</b> To an outside observer, the Zcash transaction is just encrypted noise on the blockchain, and the LoRa packet is just an encrypted RF burst. There is no mathematical way for an eavesdropper to link the two.<br/>"
        "<b>• Sustainable Open Source:</b> The 2% fee split is enforced on-chip by the gateway. If a sender tries to bypass the developer fee, the gateway's mempool scanner rejects the transaction, and the gateway refuses to route the packet. This creates a self-sustaining funding loop for your project."
    )
    story.append(Paragraph(breakthrough_text, body_style))
    
    story.append(PageBreak())
    
    # =========================================================================
    # PAGE 11: PRACTICAL USE CASE SCENARIOS
    # =========================================================================
    story.append(Paragraph("■ Practical Use Case Scenarios", h1_style))
    story.append(Spacer(1, 8))
    
    story.append(Paragraph("5.1 Scenario A: Off-Grid P2P Data Marketplace (Drone & Sensor)", h2_style))
    scenario_a_text = (
        "In this scenario, an autonomous drone (Agent-A) and a ground-based weather sensor (Agent-B) operate off-grid "
        "using only LoRa radio waves. The drone needs real-time wind speed data before landing and is willing to pay "
        "0.002 ZEC. A local internet-connected gateway acts as their Zcash network bridge, routing the transaction and earning its fee."
    )
    story.append(Paragraph(scenario_a_text, body_style))
    
    scenario_a_flow = (
        "[ Agent-A: Drone ]                 [ Agent-B: Sensor ]                [ LoRa Gateway ]                [ Zcash Blockchain ]\n"
        "   (Off-Grid)                          (Off-Grid)                        (Has Internet)                   (On-Chain)\n"
        "        │                                  │                                  │                               │\n"
        "        │ 1. Request: \"Need Wind Speed\"    │                                  │                               │\n"
        "        │ ────────────────────────────────>│                                  │                               │\n"
        "        │                                  │                                  │                               │\n"
        "        │                                  │ 2. Sends signed weather data     │                               │\n"
        "        │ <────────────────────────────────│                                  │                               │\n"
        "        │                                  │                                  │                               │\n"
        "        │ 3. Broadcasts raw Zcash TX       │                                  │                               │\n"
        "        │    - 0.00196 ZEC to Agent-B      │                                  │                               │\n"
        "        │    - 0.00004 ZEC to Dev (2%)     │                                  │                               │\n"
        "        │    - 0.00010 ZEC to Gateway      │                                  │                               │\n"
        "        │ ─────────────────────────────────┼─────────────────────────────────>│                               │\n"
        "        │                                  │                                  │ 4. Scans TX in mempool        │\n"
        "        │                                  │                                  │ 5. Verifies its own fee       │\n"
        "        │                                  │                                  │ 6. Verifies 2% Dev fee        │\n"
        "        │                                  │                                  │                               │\n"
        "        │                                  │                                  │ 7. Relays raw TX to Internet  │\n"
        "        │                                  │                                  │ ─────────────────────────────>│\n"
        "        │                                  │                                  │                               │ Distributed:\n"
        "        │                                  │                                  │                               │ - Sensor gets paid.\n"
        "        │                                  │                                  │                               │ - Gateway gets paid.\n"
        "        │                                  │                                  │                               │ - Dev gets paid.\n"
    )
    story.append(make_code_block(scenario_a_flow, styles, title="zk-lora-operator ~ data_marketplace.sh"))
    story.append(PageBreak())
    
    story.append(Paragraph("5.2 Scenario B: Private Search & Rescue Swarm Coordination", h2_style))
    scenario_b_text = (
        "A swarm of autonomous search-and-rescue UAVs needs to coordinate search grids and share target sightings in a remote "
        "mountainous area with zero cellular coverage. They use ZK-LoRa to broadcast encrypted grid updates. Because they "
        "use ZK-identity masking, an adversary cannot eavesdrop on their coordination or track the physical location of the "
        "drones by monitoring their RF signatures. They pay local relay nodes in ZEC to extend their coordination range."
    )
    story.append(Paragraph(scenario_b_text, body_style))
    story.append(Spacer(1, 15))
    
    story.append(Paragraph("5.3 Scenario C: Smart Agriculture & Environmental Health Monitoring", h2_style))
    scenario_c_text = (
        "Tens of thousands of soil moisture and wildfire detection sensors are scattered across a national forest. They use "
        "ZK-LoRa to transmit status updates. To prevent competitors or malicious actors from mapping the sensor locations and "
        "identifying vulnerable areas, the data is encrypted via ECIES and identities are masked with ZK-proofs. Gateways "
        "are incentivized to maintain high-uptime remote relays because they earn ZEC micropayments for every status packet they route."
    )
    story.append(Paragraph(scenario_c_text, body_style))
    
    story.append(PageBreak())
    
    # =========================================================================
    # PAGE 12: CRYPTOGRAPHIC SECURITY & ANTI-FRAUD ANALYSIS
    # =========================================================================
    story.append(Paragraph("■ Cryptographic Security & Anti-Fraud Analysis", h1_style))
    story.append(Spacer(1, 8))
    
    story.append(Paragraph("6.1 Physical RF Layer & Gateway Mitigations", h2_style))
    sec_details_1 = (
        "<b>Replay Protection</b>: Every ZK-proof binds a UTC timestamp and an ephemeral nonce. Gateways reject any "
        "packet outside a ±5-second window or with a duplicate nonce.<br/>"
        "<b>Sybil Spam Prevention</b>: Sending nodes must solve an RF-Proof-of-Work challenge, or present a symmetric "
        "HMAC using their registered session key (verified in &lt;1µs), protecting the ZK-SNARK engine from CPU exhaustion.<br/>"
        "<b>Lying Gateway Prevention</b>: Off-grid nodes verify block headers and Merkle paths locally (SPV). "
        "Gateways cannot forge confirmations without spending the computational power to solve Equihash PoW."
    )
    story.append(Paragraph(sec_details_1, body_style))
    story.append(Spacer(1, 6))

    story.append(Paragraph("6.2 Advanced Hardware Scams & ZKCP", h2_style))
    sec_details_2 = (
        "<b>The Gorgon Attack (Selective Dropping)</b>: A malicious gateway claims its routing fee in the mempool but drops the packet. "
        "We solve this via <b>Zero-Knowledge Proof-of-Delivery (ZK-PoD)</b>: the routing fee output is locked until the gateway presents a "
        "cryptographic receipt signed by the destination node.<br/>"
        "<b>The Eclipse Attack (Location Spoofing)</b>: Attacking nodes spoof coordinates to hijack routing. We enforce physical "
        "<b>Time-of-Flight (ToF) RTT checks</b> using the Semtech SX1302/1303 internal nanosecond clock to verify distance at the speed of light.<br/>"
        "<b>The Free Rider Attack (Mesh Black Holes)</b>: Relays drop packets to save battery. We implement <b>Neighbor Auditing</b>, "
        "where surrounding nodes passively attest to transmissions, slashing the reputation of black-hole nodes."
    )
    story.append(Paragraph(sec_details_2, body_style))
    story.append(Spacer(1, 10))
    
    sec_data = [
        [Paragraph("Attack Vector", table_hdr_style), Paragraph("Mitigation Mechanism", table_hdr_style), Paragraph("Security Guarantee", table_hdr_style)],
        [
            Paragraph("Replay Attack", table_cell_bold),
            Paragraph("Nonces + ±5s Window", table_cell_style),
            Paragraph("Duplicate packets rejected.", table_cell_style)
        ],
        [
            Paragraph("Sybil Spam", table_cell_bold),
            Paragraph("HMAC + RF-Proof-of-Work", table_cell_style),
            Paragraph("Jammers exhausted / filtered.", table_cell_style)
        ],
        [
            Paragraph("Lying Gateway", table_cell_bold),
            Paragraph("Consensus + SPV Checks", table_cell_style),
            Paragraph("Cannot forge Equihash PoW.", table_cell_style)
        ],
        [
            Paragraph("Gorgon Attack", table_cell_bold),
            Paragraph("ZK-Proof-of-Delivery", table_cell_style),
            Paragraph("No fee payout without delivery receipt.", table_cell_style)
        ],
        [
            Paragraph("Location Spoof", table_cell_bold),
            Paragraph("Time-of-Flight (ToF) RTT", table_cell_style),
            Paragraph("Physical distance verified.", table_cell_style)
        ],
        [
            Paragraph("Free Rider", table_cell_bold),
            Paragraph("Neighbor Auditing", table_cell_style),
            Paragraph("Black-hole nodes bypassed.", table_cell_style)
        ],
        [
            Paragraph("Timing Attack", table_cell_bold),
            Paragraph("Batched Shuffling / Credits", table_cell_style),
            Paragraph("Breaks temporal correlation.", table_cell_style)
        ],
    ]
    sec_table = Table(sec_data, colWidths=[110, 150, 244])
    sec_table.setStyle(TableStyle([
        ('BACKGROUND', (0,0), (-1,0), colors.HexColor("#0F172A")),
        ('GRID', (0,0), (-1,-1), 0.5, colors.HexColor("#CBD5E1")),
        ('ROWBACKGROUNDS', (0,1), (-1,-1), [colors.white, colors.HexColor("#F8FAFC")]),
        ('PADDING', (0,0), (-1,-1), 5),
        ('VALIGN', (0,0), (-1,-1), 'MIDDLE'),
    ]))
    story.append(sec_table)
    
    story.append(PageBreak())
    
    # =========================================================================
    # PAGE 13: PERFORMANCE & BANDWIDTH ANALYSIS
    # =========================================================================
    story.append(Paragraph("■ Performance & Bandwidth Analysis", h1_style))
    story.append(Spacer(1, 10))
    
    story.append(Paragraph("7.1 Link Budget, Bandwidth, & Regulatory Constraints", h2_style))
    perf_text = (
        "Because LoRa is a low-bandwidth modulation scheme operating in unlicensed Industrial, Scientific, and Medical (ISM) "
        "radio bands, packet size and regulatory compliance are critical. ZK-LoRa operates on license-free spectrum "
        "globally, including <b>US915</b> (902–928 MHz) in North America, <b>EU868</b> (863–870 MHz) in Europe (subject to "
        "a strict 1% duty cycle limit), <b>AU915</b> in South America, and <b>AS923</b> in Asia. This allows completely "
        "permissionless deployment with typical transmission ranges of <b>2 to 5 km</b> in urban areas, <b>10 to 15 km</b> "
        "in rural line-of-sight, and up to <b>30+ km</b> from high-elevation nodes (such as hilltops or drones).<br/><br/>"
        "To maximize efficiency and avoid packet fragmentation, ZK-LoRa optimizes its packet size. While the physical layer "
        "limit of Semtech transceivers is 255 bytes, standard unfragmented LoRaWAN payloads are capped between 222 and 242 bytes "
        "depending on Spreading Factor. ZK-LoRa supports an <b>Unfragmented Single-Packet Mode</b> (sub-236 bytes) by compressing "
        "the ECIES encrypted payload to 64 bytes and the Groth16 proof to 128 bytes (totaling 222 bytes with headers). For larger "
        "payloads, a <b>Dual-Fragment Assembly Protocol</b> is used to split the data into two sub-222-byte packets, avoiding "
        "airtime violations."
    )
    story.append(Paragraph(perf_text, body_style))
    story.append(Spacer(1, 10))
    
    bandwidth_data = [
        [Paragraph("Component", table_hdr_style), Paragraph("Size (Bytes)", table_hdr_style), Paragraph("Airtime @ SF9, 125kHz", table_hdr_style)],
        [Paragraph("Preamble & Header", table_cell_bold), Paragraph("28", table_cell_style), Paragraph("~80 ms", table_cell_style)],
        [Paragraph("Encrypted Payload (ECIES)", table_cell_bold), Paragraph("256", table_cell_style), Paragraph("~680 ms", table_cell_style)],
        [Paragraph("ZK-SNARK Proof (Groth16)", table_cell_bold), Paragraph("128", table_cell_style), Paragraph("~340 ms", table_cell_style)],
        [Paragraph("Total Packet", table_cell_bold), Paragraph("412", table_cell_style), Paragraph("~1.10 seconds", table_cell_style)]
    ]
    bw_table = Table(bandwidth_data, colWidths=[180, 100, 224])
    bw_table.setStyle(TableStyle([
        ('BACKGROUND', (0,0), (-1,0), colors.HexColor("#0F172A")),
        ('GRID', (0,0), (-1,-1), 0.5, colors.HexColor("#CBD5E1")),
        ('ROWBACKGROUNDS', (0,1), (-1,-1), [colors.white, colors.HexColor("#F8FAFC")]),
        ('PADDING', (0,0), (-1,-1), 8),
        ('VALIGN', (0,0), (-1,-1), 'MIDDLE'),
    ]))
    story.append(bw_table)
    story.append(Spacer(1, 20))
    
    story.append(Paragraph("7.2 Computational Overhead & Power Consumption on Edge Hardware", h2_style))
    comp_text = (
        "Edge hardware is resource-constrained. The table below shows estimated execution times for core "
        "cryptographic operations on a Raspberry Pi 4 / RAK Gateway node.<br/><br/>"
        "<b>Helium E-Waste Repurposing & Power Efficiency:</b> A significant advantage of ZK-LoRa is its ability "
        "to run on underutilized, pre-certified Helium hotspots (RAK V2, MNTD) that would otherwise become electronic waste. "
        "A standard node (Raspberry Pi 4 compute unit + Semtech SX1302/SX1303 LoRa concentrator) consumes only "
        "<b>3.5 Watts</b> in idle/routing mode. Under peak computational load—when actively generating a ZK-SNARK "
        "proof on the CPU and transmitting over the RF interface—the device draws a maximum of <b>7.5 Watts</b>. This "
        "ultra-low power profile enables completely off-grid operation powered by a small 10W solar panel and a 12V battery."
    )
    story.append(Paragraph(comp_text, body_style))
    story.append(Spacer(1, 10))
    
    comp_data = [
        [Paragraph("Operation", table_hdr_style), Paragraph("Execution Time", table_hdr_style), Paragraph("Frequency", table_hdr_style)],
        [Paragraph("Key Generation (secp256k1)", table_cell_bold), Paragraph("~100 ms", table_cell_style), Paragraph("One-time setup", table_cell_style)],
        [Paragraph("ECIES Encryption", table_cell_bold), Paragraph("~10 ms", table_cell_style), Paragraph("Per-packet", table_cell_style)],
        [Paragraph("ZK-SNARK Proof Gen (Groth16)", table_cell_bold), Paragraph("~1.2 seconds", table_cell_style), Paragraph("Per-packet", table_cell_style)],
        [Paragraph("ZK-SNARK Proof Verification", table_cell_bold), Paragraph("~50 ms", table_cell_style), Paragraph("Per-packet", table_cell_style)]
    ]
    comp_table = Table(comp_data, colWidths=[180, 140, 184])
    comp_table.setStyle(TableStyle([
        ('BACKGROUND', (0,0), (-1,0), colors.HexColor("#0F172A")),
        ('GRID', (0,0), (-1,-1), 0.5, colors.HexColor("#CBD5E1")),
        ('ROWBACKGROUNDS', (0,1), (-1,-1), [colors.white, colors.HexColor("#F8FAFC")]),
        ('PADDING', (0,0), (-1,-1), 8),
        ('VALIGN', (0,0), (-1,-1), 'MIDDLE'),
    ]))
    story.append(comp_table)
    
    story.append(PageBreak())
    
    # =========================================================================
    # PAGE 14: REAL-WORLD RANGE VALIDATION
    # =========================================================================
    story.append(Paragraph("7.3 The Real-World-Range Capabilities", h2_style))
    val_text = (
        "LoRaWAN technology is inherently eco-friendly, operating with extremely low power consumption (requiring "
        "only 3.5W to 5W) while achieving remarkable communication distances. Under clear line-of-sight conditions, "
        "these low-power signals can propagate across vast geographical spans without intermediate infrastructure.<br/><br/>"
        "To demonstrate this, real-world testing was conducted across Lake Ontario. A transmitting node "
        "located on the southern shore in New York—utilizing a <b>5W RAK miner</b> connected to a <b>13 dBi Omni-directional antenna</b> "
        "mounted on a balcony on the <b>14th floor of an apartment</b>—successfully established a direct link with a gateway located in "
        "<b>Kingston, Ontario (Canada)</b>, spanning a distance of <b>131.6 km (81.7 miles)</b>.<br/><br/>"
        "<i>Note: The left map below shows actual IoT miner packets (witnesses) transmitted over the public Helium network. "
        "The right map represents the future: the same physical link secured and encrypted using <b>ZK-LoRa</b>, protecting "
        "node identities via zero-knowledge proofs.</i>"
    )
    story.append(Paragraph(val_text, body_style))
    story.append(Spacer(1, 8))
    
    # Render the side-by-side comparison of the maps
    if os.path.exists("lake_ontario_range.png") and os.path.exists("zk_lora_gold_map.png"):
        img_width = 170
        img_height = 170 * (1661 / 1079) # ~261 pt
        img_left = Image('lake_ontario_range.png', width=img_width, height=img_height)
        img_right = Image('zk_lora_gold_map.png', width=img_width, height=img_height)
        
        map_table = Table([[img_left, img_right]], colWidths=[240, 240])
        map_table.setStyle(TableStyle([
            ('ALIGN', (0,0), (-1,-1), 'CENTER'),
            ('VALIGN', (0,0), (-1,-1), 'MIDDLE'),
            ('BOX', (0,0), (0,0), 1.5, colors.HexColor("#64748B")), # Gray border for Helium
            ('BOX', (1,0), (1,0), 1.5, colors.HexColor("#F3B300")), # Gold border for ZK-LoRa
            ('BACKGROUND', (0,0), (-1,-1), colors.HexColor("#0F172A")),
            ('BOTTOMPADDING', (0,0), (-1,-1), 6),
            ('TOPPADDING', (0,0), (-1,-1), 6),
            ('LEFTPADDING', (0,0), (-1,-1), 6),
            ('RIGHTPADDING', (0,0), (-1,-1), 6),
        ]))
        map_table.hAlign = 'CENTER'
        story.append(map_table)
        story.append(Spacer(1, 6))
        
        # Comparison section: Custom description vs Zcash logo + "This could be you now:"
        logo_right = Image('zcash_eco_recycle_logo.png', width=85, height=85) if os.path.exists('zcash_eco_recycle_logo.png') else Spacer(1, 85)
        
        comparison_data = [
            [Paragraph("<font size=9 color='#94A3B8'>This is the power of LoRaWAN via a 13 dBi Omni antenna at 146ft height only consuming 5 watts of power in a RAK miner.</font>", ParagraphStyle('LeftDesc', fontName='Helvetica-Oblique', fontSize=8.5, leading=12, alignment=0, textColor=colors.HexColor("#94A3B8"))), 
             Paragraph("<font size=10 color='#F3B300'><img src='zcash_logo.png' width='14' height='14' valign='middle'/> <b>This could be you now:</b></font>", ParagraphStyle('OrThisCouldBeYou', alignment=1))],
            [Spacer(1, 1), logo_right]
        ]
        comparison_table = Table(comparison_data, colWidths=[240, 240])
        comparison_table.setStyle(TableStyle([
            ('ALIGN', (0,0), (-1,-1), 'CENTER'),
            ('VALIGN', (0,0), (-1,-1), 'MIDDLE'),
            ('BOTTOMPADDING', (0,0), (-1,-1), 2),
            ('TOPPADDING', (0,0), (-1,-1), 2),
        ]))
        comparison_table.hAlign = 'CENTER'
        story.append(comparison_table)
        
    story.append(PageBreak())
    
    # =========================================================================
    # PAGE 14: CRYPTOGRAPHIC AUDIT & DEEP VULNERABILITY ANALYSIS
    # =========================================================================
    story.append(Paragraph("■ Cryptographic Audit & Deep Vulnerability Analysis", h1_style))
    story.append(Spacer(1, 10))
    
    audit_intro = (
        "For ZK-LoRa to achieve absolute Zcash-grade security, we must audit the underlying mathematics, "
        "cryptographic curves, and hardware implementations of our zero-knowledge systems."
    )
    story.append(Paragraph(audit_intro, body_style))
    story.append(Spacer(1, 8))
    
    story.append(Paragraph("8.1 Key Cryptographic Vulnerabilities & Mitigations", h2_style))
    
    audit_details = (
        "<b>1. Trusted Setup (Groth16)</b>: Groth16 requires a phase-2 trusted setup. If the 'toxic waste' "
        "(&tau;) is not destroyed, an attacker can forge proofs. <i>Mitigation:</i> We will conduct a public MPC "
        "ceremony (Powers of Tau) with &gt;100 participants. Long-term, we will migrate to <b>Halo2</b> (transparent setup).<br/>"
        "<b>2. Curve Security (BN254)</b>: Recent NFS advances reduce BN254's security to ~100 bits. "
        "<i>Mitigation:</i> ZK-LoRa's production spec dictates migration to <b>BLS12-381</b> (128-bit) or the "
        "Zcash-standard <b>Pasta</b> curves (Pallas/Vesta) for recursive proving.<br/>"
        "<b>3. Proof Malleability</b>: Groth16 proofs are malleable; an adversary can mutate proof bytes and replay them. "
        "<i>Mitigation:</i> We bind the proof to the transaction payload and sign the entire packet using <b>Ed25519</b>. "
        "Any mutation invalidates the signature.<br/>"
        "<b>4. Under-Constrained Circuits</b>: Missing constraints in Circom allow provers to cheat. "
        "<i>Mitigation:</i> We run all circuits through <b>Circomspect</b> static analysis and enforce strict bit-decomposition range proofs.<br/>"
        "<b>5. Side-Channel Attacks</b>: Physical access to edge nodes allows key extraction via power analysis (DPA). "
        "<i>Mitigation:</i> Node schematics mandate external <b>Secure Elements</b> (e.g., ATECC608B) to keep keys in tamper-resistant silicon."
    )
    story.append(Paragraph(audit_details, body_style))
    story.append(Spacer(1, 10))
    
    story.append(NextPageTemplate('Last'))
    story.append(PageBreak())
    
    # =========================================================================
    # PAGE 15: SPECIAL THANKS & ACKNOWLEDGEMENTS + THE AI COLLECTIVE LOGO (BLACK PAGE)
    # =========================================================================
    story.append(Spacer(1, 20))
    thanks_style = ParagraphStyle(
        'ThanksStyle',
        fontName='Helvetica-Oblique',
        fontSize=11,
        leading=16,
        textColor=colors.HexColor("#CBD5E1"),
        alignment=1
    )
    story.append(Paragraph(
        "Special thanks to the Zcash Community Grants committee and the Zcash Foundation "
        "for supporting privacy-preserving decentralized infrastructure and promoting zero-knowledge "
        "research at the edge.",
        thanks_style
    ))
    story.append(Spacer(1, 15))
    
    disclaimer_long = (
        "This whitepaper and proposal are intended for educational and project evaluation purposes only. "
        "This ZK-LoRa codebase is currently pending to be released under the MIT License upon approval of the Grant."
    )
    story.append(Paragraph(disclaimer_long, ParagraphStyle('DisclaimerLong', alignment=1, fontSize=8.5, leading=13, textColor=colors.HexColor("#94A3B8"))))
    story.append(Spacer(1, 20))
    
    # Add both logos side-by-side on Page 16
    logo_left_p16 = Image('zcash_logo.png', width=40, height=40) if os.path.exists('zcash_logo.png') else Spacer(1, 40)
    logo_right_p16 = Image('zcash_eco_recycle_logo.png', width=50, height=50) if os.path.exists('zcash_eco_recycle_logo.png') else Spacer(1, 50)
    
    p16_logo_data = [
        [logo_left_p16, logo_right_p16],
        [Paragraph("<font size=7 color='#94A3B8'>Standard Zcash Logo</font>", caption_style),
         Paragraph("<font size=7 color='#94A3B8'>Proposed Eco-Recycle Logo<br/>(Upon Grant Approval)</font>", caption_style)]
    ]
    p16_logo_table = Table(p16_logo_data, colWidths=[120, 120])
    p16_logo_table.setStyle(TableStyle([
        ('ALIGN', (0,0), (-1,-1), 'CENTER'),
        ('VALIGN', (0,0), (-1,-1), 'MIDDLE'),
        ('BOTTOMPADDING', (0,0), (-1,-1), 2),
        ('TOPPADDING', (0,0), (-1,-1), 2),
    ]))
    p16_logo_table.hAlign = 'CENTER'
    story.append(p16_logo_table)
    story.append(Spacer(1, 10))
        
    story.append(Paragraph("<font size='14' color='#FFFFFF'><b>WE ARE</b></font>", ParagraphStyle('WeAre', alignment=1)))
    story.append(Spacer(1, 10))
    
    logos_style = ParagraphStyle('Logos', fontName='Helvetica-Bold', fontSize=14, leading=18, textColor=colors.HexColor("#F3B300"), alignment=1)
    story.append(Paragraph("THE AI COLLECTIVE", logos_style))
    story.append(Spacer(1, 20))

    if os.path.exists("theaicollective_logo.jpeg"):
        story.append(Image("theaicollective_logo.jpeg", width=240, height=240, hAlign='CENTER'))
    else:
        story.append(Spacer(1, 240))
        
    story.append(Spacer(1, 15))
    quote_style = ParagraphStyle(
        'CoverQuote',
        fontName='Helvetica-BoldOblique',
        fontSize=10.5,
        leading=15,
        textColor=colors.HexColor("#F3B300"),
        alignment=1
    )
    story.append(Paragraph(
        "\"The impossible is just code waiting to be written, physics waiting to be rewritten,<br/>"
        "math a work in progress, and truth waiting to be discovered.\"",
        quote_style
    ))
        
    # Build the document
    doc.build(story, canvasmaker=NumberedCanvas)
    print("PDF Generation complete: ZK_LoRa_Whitepaper.pdf")

if __name__ == "__main__":
    build_pdf()
