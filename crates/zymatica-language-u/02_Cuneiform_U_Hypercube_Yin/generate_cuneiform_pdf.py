# -*- coding: utf-8 -*-
import os
import re
import shutil
from reportlab.lib.pagesizes import letter
from reportlab.platypus import SimpleDocTemplate, Paragraph, Spacer, Table, TableStyle, Image, PageBreak, KeepTogether
from reportlab.lib.styles import getSampleStyleSheet, ParagraphStyle
from reportlab.lib import colors
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
            self.draw_page_decorations(num_pages)
            super().showPage()
        super().save()

    def draw_page_decorations(self, page_count):
        self.saveState()
        # Running Header
        self.setFont("Helvetica-Bold", 8)
        self.setFillColor(colors.HexColor("#0F2D59"))
        self.drawString(54, 755, "ZYMATICA | CUNEIFORM-U 6D SEMANTIC HYPERCUBE")
        
        self.setFont("Helvetica", 8)
        self.setFillColor(colors.HexColor("#718096"))
        self.drawRightString(558, 755, "CLASS 02: 6D COORDINATE METRIC MANIFOLD")
        
        # Header line
        self.setStrokeColor(colors.HexColor("#CBD5E0"))
        self.setLineWidth(0.75)
        self.line(54, 747, 558, 747)
        
        # Running Footer
        self.setStrokeColor(colors.HexColor("#CBD5E0"))
        self.setLineWidth(0.75)
        self.line(54, 55, 558, 55)
        
        self.setFont("Helvetica", 8)
        self.setFillColor(colors.HexColor("#718096"))
        self.drawString(54, 42, "zymatica.space | astronautshe.com | Devs One | We Are TheAiCollective.art")
        
        page_text = f"Page {self._pageNumber} of {page_count}"
        self.drawRightString(558, 42, page_text)
        
        self.restoreState()

def clean_md_text(text):
    text = text.replace("&", "&amp;").replace("<", "&lt;").replace(">", "&gt;")
    text = re.sub(r'\$\$(.*?)\$\$', r'<i>\1</i>', text)
    text = re.sub(r'\$(.*?)\$', r'<i>\1</i>', text)
    text = re.sub(r'\*\*(.*?)\*\*', r'<b>\1</b>', text)
    text = re.sub(r'__(.*?)__', r'<b>\1</b>', text)
    text = re.sub(r'\*(.*?)\*', r'<i>\1</i>', text)
    text = re.sub(r'`(.*?)`', r'<b><font name="Courier" color="#0F2D59">\1</font></b>', text)
    text = re.sub(r'\[(.*?)\]\((.*?)\)', r'<a href="\2"><font color="#1D4ED8"><b>\1</b></font></a>', text)
    return text

def generate_pdf(output_pdf_path="WHITEPAPER.pdf"):
    doc = SimpleDocTemplate(
        output_pdf_path,
        pagesize=letter,
        leftMargin=54,
        rightMargin=54,
        topMargin=60,
        bottomMargin=65
    )
    
    styles = getSampleStyleSheet()
    
    title_style = ParagraphStyle(
        'DocTitle',
        parent=styles['Heading1'],
        fontName='Helvetica-Bold',
        fontSize=20,
        leading=24,
        textColor=colors.HexColor("#0F2D59"),
        spaceAfter=6
    )
    
    subtitle_style = ParagraphStyle(
        'DocSubtitle',
        parent=styles['Normal'],
        fontName='Helvetica-Oblique',
        fontSize=10,
        leading=14,
        textColor=colors.HexColor("#4A5568"),
        spaceAfter=14
    )
    
    h2_style = ParagraphStyle(
        'DocH2',
        parent=styles['Heading2'],
        fontName='Helvetica-Bold',
        fontSize=13,
        leading=17,
        textColor=colors.HexColor("#0F2D59"),
        spaceBefore=14,
        spaceAfter=8
    )
    
    h3_style = ParagraphStyle(
        'DocH3',
        parent=styles['Heading3'],
        fontName='Helvetica-Bold',
        fontSize=10.5,
        leading=14,
        textColor=colors.HexColor("#2B6CB0"),
        spaceBefore=10,
        spaceAfter=4
    )
    
    body_style = ParagraphStyle(
        'DocBody',
        parent=styles['Normal'],
        fontName='Helvetica',
        fontSize=9,
        leading=13.5,
        textColor=colors.HexColor("#2D3748"),
        spaceAfter=8
    )
    
    quote_style = ParagraphStyle(
        'DocQuote',
        parent=styles['Normal'],
        fontName='Helvetica-Oblique',
        fontSize=9.5,
        leading=14,
        textColor=colors.HexColor("#1A365D"),
        spaceBefore=6,
        spaceAfter=12
    )
    
    story = []
    
    story.append(Paragraph("ZYMATICA: Cuneiform-U Semantic Hypercube System", title_style))
    story.append(Paragraph("<b>IP Class 02 &nbsp;|&nbsp; 6D Coordinate Metric Manifold &nbsp;|&nbsp; Apache License 2.0</b>", subtitle_style))
    
    story.append(Paragraph("<i>\"The impossible is just code waiting to be written, physics waiting to be rewritten, math a work in progress, and truth waiting to be discovered.\"</i>", quote_style))
    story.append(Spacer(1, 10))
    
    story.append(Paragraph("1. Technical Overview &amp; Mathematical Framework", h2_style))
    story.append(Paragraph(
        "The <b>Cuneiform-U Semantic Hypercube</b> is a structured coordinate metric space that maps discrete natural language tokens onto a continuous, low-dimensional geometric manifold.",
        body_style
    ))
    story.append(Paragraph(
        "Traditional tokenizers represent vocabulary items as unstructured, flat integers (e.g., Token ID 48102). In low-rank weight projections (SVD compression), quantization noise shatters the model's logit distribution, leading to catastrophic syntactic collapse where the model generates random, out-of-vocabulary characters.",
        body_style
    ))
    story.append(Paragraph(
        "Cuneiform-U solves this by mapping all <i>N</i> tokens in the vocabulary into a <b>6-Dimensional Hypercube</b> along six orthogonal semantic axes:",
        body_style
    ))
    
    axes = [
        ("1. Domain (D)", "The macro-topic category (0-15; e.g., Hardware, Math, Dialogue, Software, General)."),
        ("2. Subdomain (S)", "The micro-topic context (0-15; e.g., LoRa networks, GPIO, SVD projection, Entropy, Python, Rust)."),
        ("3. Operation (O)", "The functional action or state transition (0-15; e.g., reset, write, compress, heal, grow)."),
        ("4. Modality (M)", "The data format, layout, or syntax type (0-15; e.g., binary, json, packet, byte, token)."),
        ("5. Depth (d)", "The complexity hierarchy or scale (0-15; e.g., seeds, atoms, factoids)."),
        ("6. Polarity (P)", "The outcome direction or flag (0-15; e.g., ACK, NACK, success, fail, neutral)."),
    ]
    
    for axis, desc in axes:
        story.append(Paragraph(f"• <b>{axis}:</b> {desc}", body_style))
        
    story.append(Spacer(1, 8))
    story.append(Paragraph("Radical Packing Scheme (3-Byte Wire Format)", h3_style))
    story.append(Paragraph(
        "To compress these 6 coordinate nibbles (24 bits total / 3 bytes) for ultra-low bandwidth channels (e.g., 915 MHz LoRa RF), the values are packed into three 8-bit <b>Radical Bytes</b>:",
        body_style
    ))
    story.append(Paragraph("• <b>Classifier Radical (R<sub>C</sub>):</b> Encodes high-level taxonomy: <i>R<sub>C</sub> = (D &lt;&lt; 4) | (S &amp; 0xF)</i>", body_style))
    story.append(Paragraph("• <b>Factor Radical (R<sub>F</sub>):</b> Encodes system action and modality: <i>R<sub>F</sub> = (O &lt;&lt; 4) | (M &amp; 0xF)</i>", body_style))
    story.append(Paragraph("• <b>Active Radical (R<sub>A</sub>):</b> Encodes depth complexity and logical polarity: <i>R<sub>A</sub> = (d &lt;&lt; 4) | (P &amp; 0xF)</i>", body_style))
    
    story.append(Spacer(1, 10))
    story.append(Paragraph("2. Adversarial Peer Audit: Critiques &amp; Mathematical Defenses", h2_style))
    
    story.append(Paragraph("Critique 2.1: Semantic Compression Ambiguity (Many-to-One)", h3_style))
    story.append(Paragraph("<b>The Skeptic's View:</b> Why map tokens to 6D coordinates? If the vocabulary size (256,000 tokens) fits within the 24-bit space (16.7 million states), you have a bijective mapping. Why not just run a standard Neural Arithmetic Coder on token IDs?", body_style))
    story.append(Paragraph("<b>The Mathematical Defense:</b> This is the core novelty of the hypercube. If you compress a flat vocabulary using a standard neural arithmetic coder, the model treats token IDs as independent classes. Under quantization noise (SVD degradation), the model's logits drift, causing standard arithmetic coding to fail catastrophically because the model predicts a completely random, out-of-vocabulary token. By mapping tokens to a 6D semantic metric space (Cuneiform-U), tokens that are semantically similar are placed close to each other geometrically. During SFT, the Radical Coordinate Resonance Loss (RCRA) optimizes the model using the geometric distance between predicted coordinates. If the model makes an error under heavy compression, the loss forces it to output a token that is semantically close (neighboring coordinates) rather than a syntactic hallucination. Furthermore, the 6D axes enable the S-PAUP router to JIT-swap adapters on the GPU by checking coordinate bounds.", body_style))
    
    story.append(Spacer(1, 8))
    story.append(Paragraph("Critique 2.2: Channel Entropy &amp; Physical Feasibility", h3_style))
    story.append(Paragraph("<b>The Skeptic's View:</b> Does this violate Claude Shannon's source coding theorem?", body_style))
    story.append(Paragraph("<b>The Mathematical Defense:</b> No. Claude Shannon's 1948 Source Coding Theorem explicitly set semantic meaning aside to address syntactic character transmission over noisy channels: <i>H(X) = -Σ P(xᵢ) · log₂(P(xᵢ))</i>. Language-U decomposes information into <i>H(Text) = H(Meaning) + H(Syntax | Meaning)</i>. The physical channel only carries the 24-bit geometric trajectory through the synchronized 6D prior manifold, achieving 7x to 10x bandwidth reduction over raw character transport while strictly respecting Shannon's conditional entropy bounds.", body_style))
    
    doc.build(story, canvasmaker=NumberedCanvas)
    print(f"Generated {output_pdf_path} successfully!")

if __name__ == "__main__":
    generate_pdf("WHITEPAPER.pdf")
