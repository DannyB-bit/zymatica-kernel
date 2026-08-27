# -*- coding: utf-8 -*-
import os
import re
import sys
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
        # Page 1: Full-page Language U Logo
        if self._pageNumber == 1:
            logo_path = "language_u_logo.jpg" if os.path.exists("language_u_logo.jpg") else os.path.join(os.path.dirname(os.path.abspath(__file__)), "language_u_logo.jpg")
            if os.path.exists(logo_path):
                self.drawImage(logo_path, 0, 0, width=letter[0], height=letter[1])
        # Last Page: Full-page AI Collective Logo with custom sign-off
        elif self._pageNumber == page_count:
            logo_path = "Logo.jpg" if os.path.exists("Logo.jpg") else os.path.join(os.path.dirname(os.path.abspath(__file__)), "Logo.jpg")
            if os.path.exists(logo_path):
                self.drawImage(logo_path, 0, 0, width=letter[0], height=letter[1])
            
            # Semi-transparent overlay at the bottom for the sign-off
            self.setFillColor(colors.HexColor("#0D1527"))
            self.rect(0, 0, letter[0], 85, fill=True, stroke=False)
            
            self.setFont("Helvetica-Bold", 10)
            self.setFillColor(colors.white)
            self.drawCentredString(letter[0]/2.0, 48, "zymatica.space  |  astronautshe.com  |  Devs One")
            self.setFont("Helvetica-Bold", 8)
            self.setFillColor(colors.HexColor("#63B3ED"))
            self.drawCentredString(letter[0]/2.0, 28, "We Are TheAiCollective.art  Apache License 2.0 2026©")
        # Middle Pages (Running Headers & Footers)
        else:
            # Running Header
            self.setFont("Helvetica-Bold", 8)
            self.setFillColor(colors.HexColor("#1A365D"))
            self.drawString(54, 755, "ZYMATICA VOICE: A GUIDE TO LORA FOR AI AGENTS")
            
            self.setFont("Helvetica", 8)
            self.setFillColor(colors.HexColor("#718096"))
            self.drawRightString(558, 755, "IP CLASS 05/10 - TECHNICAL SPECIFICATION")
            
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
            self.drawString(54, 42, "© 2026 Zymatica.space | astronautshe.com | Devs One | We Are TheAiCollective.art")
            
            page_text = f"Page {self._pageNumber} of {page_count}"
            self.drawRightString(558, 42, page_text)
            
        self.restoreState()

def md_to_html(text):
    # Escape '&' but avoid double escaping if it's already an entity
    # A simple way is to replace '&' with '&amp;' except for &lt;, &gt;, &amp;, &bull;, &ndash;, &mdash;
    # Let's replace '&' first
    text = text.replace("&", "&amp;")
    text = text.replace("&amp;amp;", "&amp;")
    text = text.replace("&amp;bull;", "&bull;")
    text = text.replace("&amp;ndash;", "&ndash;")
    text = text.replace("&amp;mdash;", "&mdash;")
    
    # Replace '<' and '>' except when they look like HTML tags we want to support:
    # <b>, </b>, <i>, </i>, <font ...>, </font>, <a>, </a>, <br/>, <br>
    # We can temporarily hide our tags, clean the rest, and restore them, or just use regular expressions carefully.
    
    # Let's do markdown replacement
    # Bold **text**
    text = re.sub(r'\*\*(.*?)\*\*', r'<b>\1</b>', text)
    # Italic *text*
    text = re.sub(r'\*(.*?)\*', r'<i>\1</i>', text)
    # Inline code `text`
    text = re.sub(r'`(.*?)`', r'<font face="Courier" color="#2C5282">\1</font>', text)
    # Links [text](url)
    text = re.sub(r'\[(.*?)\]\((.*?)\)', r'<a href="\2"><font color="#2B6CB0"><u>\1</u></font></a>', text)
    
    return text

def parse_markdown(filepath):
    if not os.path.exists(filepath):
        print(f"Error: {filepath} not found.")
        sys.exit(1)
        
    with open(filepath, 'r', encoding='utf-8') as f:
        lines = f.readlines()
        
    blocks = []
    current_block = None
    
    in_code = False
    code_lang = ""
    code_lines = []
    
    in_table = False
    table_lines = []
    
    in_quote = False
    quote_lines = []
    
    for line_raw in lines:
        line = line_raw.rstrip('\r\n')
        line_stripped = line.strip()
        
        # Code block handler
        if line_stripped.startswith('```'):
            if in_code:
                # End of code block
                blocks.append({
                    'type': 'code',
                    'lang': code_lang,
                    'content': '\n'.join(code_lines)
                })
                in_code = False
                code_lines = []
            else:
                # Start of code block
                in_code = True
                code_lang = line_stripped[3:].strip()
            continue
            
        if in_code:
            code_lines.append(line)
            continue
            
        # Table handler
        if line_stripped.startswith('|'):
            if not in_table:
                in_table = True
                table_lines = []
            table_lines.append(line)
            continue
        elif in_table:
            # Table ended
            blocks.append({
                'type': 'table',
                'content': table_lines
            })
            in_table = False
            table_lines = []
            
        # Blockquote handler
        if line_stripped.startswith('>'):
            if not in_quote:
                in_quote = True
                quote_lines = []
            # Strip the leading '>' and space
            content = line_stripped[1:].strip()
            quote_lines.append(content)
            continue
        elif in_quote:
            # Blockquote ended
            blocks.append({
                'type': 'quote',
                'content': '\n'.join(quote_lines)
            })
            in_quote = False
            quote_lines = []
            
        # Bullet list item handler
        if line_stripped.startswith('* ') or line_stripped.startswith('- ') or re.match(r'^\d+\.\s', line_stripped):
            is_ordered = bool(re.match(r'^\d+\.\s', line_stripped))
            if is_ordered:
                match = re.match(r'^(\d+)\.\s(.*)', line_stripped)
                num = match.group(1)
                text = match.group(2)
                blocks.append({
                    'type': 'list_item',
                    'ordered': True,
                    'number': num,
                    'content': text
                })
            else:
                text = line_stripped[2:]
                blocks.append({
                    'type': 'list_item',
                    'ordered': False,
                    'content': text
                })
            continue
            
        # Headers
        if line_stripped.startswith('# '):
            blocks.append({'type': 'h1', 'content': line_stripped[2:]})
            continue
        elif line_stripped.startswith('## '):
            blocks.append({'type': 'h2', 'content': line_stripped[3:]})
            continue
        elif line_stripped.startswith('### '):
            blocks.append({'type': 'h3', 'content': line_stripped[4:]})
            continue
            
        # Horizontal rule
        if line_stripped in ['---', '***']:
            blocks.append({'type': 'hr'})
            continue
            
        # Empty lines
        if not line_stripped:
            continue
            
        # Standard paragraph
        blocks.append({'type': 'paragraph', 'content': line_stripped})
        
    # Flush remaining blocks
    if in_code:
        blocks.append({'type': 'code', 'lang': code_lang, 'content': '\n'.join(code_lines)})
    if in_table:
        blocks.append({'type': 'table', 'content': table_lines})
    if in_quote:
        blocks.append({'type': 'quote', 'content': '\n'.join(quote_lines)})
        
    return blocks

def build_pdf(md_path, pdf_path):
    print(f"Parsing markdown from: {md_path}")
    blocks = parse_markdown(md_path)
    
    doc = SimpleDocTemplate(
        pdf_path,
        pagesize=letter,
        leftMargin=54,
        rightMargin=54,
        topMargin=72,
        bottomMargin=72
    )
    
    styles = getSampleStyleSheet()
    
    # Custom Palette
    primary_color = colors.HexColor("#1A365D")   # Deep Navy
    secondary_color = colors.HexColor("#2B6CB0") # Slate Blue
    dark_neutral = colors.HexColor("#2D3748")    # Charcoal
    accent_color = colors.HexColor("#9B2C2C")    # Deep Crimson
    light_bg = colors.HexColor("#F7FAFC")        # Warm White
    border_color = colors.HexColor("#E2E8F0")    # Border Grey
    
    # Custom Styles
    title_style = ParagraphStyle(
        'DocTitle',
        parent=styles['Heading1'],
        fontName='Helvetica-Bold',
        fontSize=20,
        leading=24,
        textColor=primary_color,
        spaceAfter=4
    )
    
    subtitle_style = ParagraphStyle(
        'DocSubtitle',
        parent=styles['Normal'],
        fontName='Helvetica',
        fontSize=11,
        leading=15,
        textColor=secondary_color,
        spaceAfter=12
    )
    
    meta_style = ParagraphStyle(
        'DocMeta',
        parent=styles['Normal'],
        fontName='Helvetica-Bold',
        fontSize=9.5,
        leading=13,
        textColor=dark_neutral,
        spaceAfter=2
    )
    
    h1_style = ParagraphStyle(
        'SecHeading1',
        parent=styles['Heading1'],
        fontName='Helvetica-Bold',
        fontSize=13.5,
        leading=17,
        textColor=primary_color,
        spaceBefore=14,
        spaceAfter=8,
        keepWithNext=True
    )
    
    h2_style = ParagraphStyle(
        'SecHeading2',
        parent=styles['Heading2'],
        fontName='Helvetica-Bold',
        fontSize=10.5,
        leading=14,
        textColor=secondary_color,
        spaceBefore=10,
        spaceAfter=6,
        keepWithNext=True
    )
    
    h3_style = ParagraphStyle(
        'SecHeading3',
        parent=styles['Heading3'],
        fontName='Helvetica-Bold',
        fontSize=9.5,
        leading=13,
        textColor=dark_neutral,
        spaceBefore=8,
        spaceAfter=4,
        keepWithNext=True
    )
    
    body_style = ParagraphStyle(
        'BodyText',
        parent=styles['Normal'],
        fontName='Helvetica',
        fontSize=9,
        leading=13,
        textColor=dark_neutral,
        spaceAfter=6
    )
    
    bullet_style = ParagraphStyle(
        'BulletText',
        parent=styles['Normal'],
        fontName='Helvetica',
        fontSize=8.5,
        leading=12.5,
        textColor=dark_neutral,
        leftIndent=15,
        firstLineIndent=-10,
        spaceAfter=3
    )
    
    code_style = ParagraphStyle(
        'CodeText',
        parent=styles['Normal'],
        fontName='Courier',
        fontSize=7.5,
        leading=10,
        textColor=colors.HexColor("#2C5282")
    )
    
    quote_style = ParagraphStyle(
        'QuoteText',
        parent=styles['Normal'],
        fontName='Helvetica-Oblique',
        fontSize=8.5,
        leading=12,
        textColor=colors.HexColor("#2D3748")
    )
    
    table_header_style = ParagraphStyle(
        'TableHeader',
        parent=styles['Normal'],
        fontName='Helvetica-Bold',
        fontSize=8,
        leading=11,
        textColor=colors.white
    )
    
    table_cell_style = ParagraphStyle(
        'TableCell',
        parent=styles['Normal'],
        fontName='Helvetica',
        fontSize=7.5,
        leading=10.5,
        textColor=dark_neutral
    )
    
    table_cell_bold = ParagraphStyle(
        'TableCellBold',
        parent=styles['Normal'],
        fontName='Helvetica-Bold',
        fontSize=7.5,
        leading=10.5,
        textColor=dark_neutral
    )
    
    story = []
    
    # Front cover page placeholder
    story.append(Spacer(1, 10))
    story.append(PageBreak())
    
    # --- COVER PAGE ---
    # Header branding table
    logo_path = "Logo_Zymatica_Voice.png"
    if not os.path.exists(logo_path):
        logo_path = "../Logo_Zymatica_Voice.png"
    if not os.path.exists(logo_path):
        # Fallback to J:\Language-U path
        logo_path = os.path.join(os.path.dirname(os.path.abspath(__file__)), "Logo_Zymatica_Voice.png")
    if not os.path.exists(logo_path):
        logo_path = os.path.join(os.path.dirname(os.path.abspath(__file__)), "Logo_Zymatica_Voice.png")
        
    logo_exists = os.path.exists(logo_path)
    
    header_data = []
    if logo_exists:
        logo_img = Image(logo_path, width=54, height=54)
        header_data = [[logo_img, Paragraph("<b>THE AI COLLECTIVE</b><br/><font color='#718096'>zymatica.space &bull; astronautshe.com &bull; Devs One</font>", subtitle_style)]]
    else:
        header_data = [[Paragraph("<b>THE AI COLLECTIVE</b><br/><font color='#718096'>zymatica.space &bull; astronautshe.com &bull; Devs One</font>", subtitle_style)]]
        
    header_table = Table(header_data, colWidths=[65, 439] if logo_exists else [504])
    header_table.setStyle(TableStyle([
        ('VALIGN', (0,0), (-1,-1), 'MIDDLE'),
        ('BOTTOMPADDING', (0,0), (-1,-1), 10),
    ]))
    story.append(header_table)
    story.append(Spacer(1, 10))
    
    # Separator Line
    story.append(Table([[ "" ]], colWidths=[504], rowHeights=[2], style=TableStyle([
        ('BACKGROUND', (0,0), (-1,-1), primary_color),
        ('BOTTOMPADDING', (0,0), (-1,-1), 0),
        ('TOPPADDING', (0,0), (-1,-1), 0),
    ])))
    story.append(Spacer(1, 15))
    
    # Document Title & Info
    story.append(Paragraph("zymatica voice - A Guide To LoRa", title_style))
    story.append(Paragraph("Zymatica Proprietary Protocol Specification | IP Class 05/10", subtitle_style))
    
    story.append(Paragraph("<b>Prepared For:</b> Autonomous AI Systems Agents, Coding Copilots, and Orchestrators", meta_style))
    story.append(Paragraph("<b>Mission:</b> Execute, debug, and verify Language-U Semantic Communication loops over lossy physical LoRa channels.", meta_style))
    story.append(Paragraph("<b>Authors & The AI Collective:</b> zymatica.space | astronautshe.com | Devs One (We Are TheAiCollective.art)", meta_style))
    story.append(Spacer(1, 20))
    
    # Large Cover Logo Image if available
    if logo_exists:
        large_logo = Image(logo_path, width=160, height=160)
        story.append(KeepTogether([
            Spacer(1, 15),
            Table([[large_logo]], colWidths=[504], style=TableStyle([
                ('ALIGN', (0,0), (-1,-1), 'CENTER'),
                ('BOTTOMPADDING', (0,0), (-1,-1), 15),
            ])),
            Spacer(1, 15),
        ]))
        
    # Abstract Box or Callout Box at bottom of Page 1
    notice_text = (
        "<b>SYSTEM PROFILE SUMMARY:</b> This specification provides the complete hardware integration, "
        "driver adjustments, validation parameters, and diagnostic routines for the RAKWireless RAK2287/RAK5146 gateway "
        "concentrator modules based on the Semtech SX1302 LoRa baseband chip. It outlines reset sequences, I2C temperature sensor patches, "
        "and dynamic verification scripts using LLD-AC range-coding and XOR-FEC packetization. Designed for direct parser parsing."
    )
    notice_table = Table([[ Paragraph(notice_text, table_cell_style) ]], colWidths=[504])
    notice_table.setStyle(TableStyle([
        ('BACKGROUND', (0,0), (-1,-1), colors.HexColor("#EDF2F7")),
        ('BORDER', (0,0), (-1,-1), 0.75, colors.HexColor("#CBD5E0")),
        ('PADDING', (0,0), (-1,-1), 10),
    ]))
    story.append(notice_table)
    
    story.append(PageBreak())
    
    # --- PARSING CONTENT ---
    # We will build the remaining document sections
    idx = 0
    while idx < len(blocks):
        block = blocks[idx]
        b_type = block['type']
        
        if b_type == 'h1':
            # We don't repeat the main page 1 title, but if it's there we can render it.
            # Skip if it is the title since we did cover page
            if "zymatica voice" in block['content'].lower():
                idx += 1
                continue
            text = md_to_html(block['content'])
            story.append(Paragraph(text, h1_style))
            
        elif b_type == 'h2':
            # Skip branding headers already handled on cover
            if "we are theaicollective.art" in block['content'].lower():
                idx += 1
                continue
            text = md_to_html(block['content'])
            story.append(Paragraph(text, h2_style))
            
        elif b_type == 'h3':
            text = md_to_html(block['content'])
            story.append(Paragraph(text, h3_style))
            
        elif b_type == 'paragraph':
            # Skip licensing subheadings that belong to cover metadata
            if "ip class 05/10" in block['content'].lower():
                idx += 1
                continue
            text = md_to_html(block['content'])
            story.append(Paragraph(text, body_style))
            
        elif b_type == 'list_item':
            text = md_to_html(block['content'])
            if block['ordered']:
                bullet_prefix = f"<b>{block['number']}.</b> "
                story.append(Paragraph(f"{bullet_prefix}{text}", bullet_style))
            else:
                bullet_prefix = "&bull; "
                story.append(Paragraph(f"{bullet_prefix}{text}", bullet_style))
                
        elif b_type == 'code':
            # Preformatted code blocks
            code_content = block['content']
            # Escape HTML characters so reportlab doesn't break
            code_content = code_content.replace("&", "&amp;").replace("<", "&lt;").replace(">", "&gt;")
            
            # Format text into Paragraphs to support wrap-around (or pre-formatting style)
            code_lines_flow = []
            for c_line in code_content.splitlines():
                # Retain indentation by replacing spaces with non-breaking spaces
                c_line_indented = c_line.replace(" ", "&nbsp;")
                code_lines_flow.append(Paragraph(c_line_indented, code_style))
                
            # Render code in a grey box Table
            code_box_table = Table([[code_lines_flow]], colWidths=[504])
            code_box_table.setStyle(TableStyle([
                ('BACKGROUND', (0,0), (-1,-1), colors.HexColor("#F7FAFC")),
                ('BORDER', (0,0), (-1,-1), 0.5, colors.HexColor("#CBD5E0")),
                ('PADDING', (0,0), (-1,-1), 8),
                ('TOPPADDING', (0,0), (-1,-1), 6),
                ('BOTTOMPADDING', (0,0), (-1,-1), 6),
            ]))
            
            story.append(KeepTogether([
                Spacer(1, 4),
                code_box_table,
                Spacer(1, 6)
            ]))
            
        elif b_type == 'quote':
            quote_text = block['content']
            
            # Check if this is a caution box
            is_caution = False
            if "[!CAUTION]" in quote_text:
                is_caution = True
                quote_text = quote_text.replace("[!CAUTION]", "").strip()
                
            quote_html = md_to_html(quote_text)
            quote_para = Paragraph(quote_html, quote_style)
            
            # Style the quote callout
            if is_caution:
                bg_col = colors.HexColor("#FFF5F5") # Reddish Alert
                brd_col = colors.HexColor("#FEB2B2")
                lbl_para = Paragraph("<b>⚠️ CAUTION: ANTENNA LOAD REQUIREMENT</b>", ParagraphStyle(
                    'CautionLabel',
                    parent=styles['Normal'],
                    fontName='Helvetica-Bold',
                    fontSize=8.5,
                    leading=12,
                    textColor=accent_color,
                    spaceAfter=4
                ))
                quote_content_table = Table([[lbl_para], [quote_para]], colWidths=[490])
            else:
                bg_col = colors.HexColor("#EDF2F7") # Greyish Info
                brd_col = colors.HexColor("#CBD5E0")
                quote_content_table = Table([[quote_para]], colWidths=[490])
                
            quote_content_table.setStyle(TableStyle([
                ('PADDING', (0,0), (-1,-1), 0),
                ('VALIGN', (0,0), (-1,-1), 'TOP'),
            ]))
            
            # Box wrapper with left accent border
            quote_box = Table([[quote_content_table]], colWidths=[504])
            quote_box.setStyle(TableStyle([
                ('BACKGROUND', (0,0), (-1,-1), bg_col),
                ('LINELEFT', (0,0), (0,-1), 4, accent_color if is_caution else secondary_color),
                ('PADDING', (0,0), (-1,-1), 8),
                ('TOPPADDING', (0,0), (-1,-1), 8),
                ('BOTTOMPADDING', (0,0), (-1,-1), 8),
                ('BORDER', (0,0), (-1,-1), 0.5, brd_col),
            ]))
            
            story.append(KeepTogether([
                Spacer(1, 6),
                quote_box,
                Spacer(1, 6)
            ]))
            
        elif b_type == 'table':
            # Parse MD table lines
            table_lines = block['content']
            
            # Filter separator lines like |:---|---|
            filtered_rows = []
            for r_line in table_lines:
                if re.match(r'^\|\s*[:\-]+\s*\|', r_line.strip()) or '---' in r_line:
                    continue
                filtered_rows.append(r_line)
                
            table_cells_data = []
            for row_idx, r_line in enumerate(filtered_rows):
                # Split cells, ignore first and last empty splits because of starting/ending |
                cells = [c.strip() for c in r_line.split('|')]
                if len(cells) > 1:
                    # If line starts and ends with |, the split list has empty cells at boundaries
                    if cells[0] == '':
                        cells = cells[1:]
                    if len(cells) > 0 and cells[-1] == '':
                        cells = cells[:-1]
                        
                row_cells_flow = []
                for cell in cells:
                    cell_html = md_to_html(cell)
                    if row_idx == 0:
                        row_cells_flow.append(Paragraph(cell_html, table_header_style))
                    else:
                        # Decide if bold cell
                        if cell.startswith('**') or cell.startswith('`'):
                            row_cells_flow.append(Paragraph(cell_html, table_cell_bold))
                        else:
                            row_cells_flow.append(Paragraph(cell_html, table_cell_style))
                if row_cells_flow:
                    table_cells_data.append(row_cells_flow)
                    
            # Check number of columns to determine widths
            if table_cells_data:
                num_cols = len(table_cells_data[0])
                # Distribute widths: 504 pt total
                if num_cols == 3:
                    # failure signature table: Error (110pt), Root Cause (120pt), Action (274pt)
                    col_widths = [110, 120, 274]
                else:
                    col_widths = [504 / num_cols] * num_cols
                    
                md_table = Table(table_cells_data, colWidths=col_widths, repeatRows=1)
                md_table.setStyle(TableStyle([
                    ('BACKGROUND', (0,0), (-1,0), primary_color),
                    ('ALIGN', (0,0), (-1,-1), 'LEFT'),
                    ('VALIGN', (0,0), (-1,-1), 'TOP'),
                    ('BOTTOMPADDING', (0,0), (-1,-1), 5),
                    ('TOPPADDING', (0,0), (-1,-1), 5),
                    ('LEFTPADDING', (0,0), (-1,-1), 5),
                    ('RIGHTPADDING', (0,0), (-1,-1), 5),
                    ('ROWBACKGROUNDS', (0,1), (-1,-1), [colors.white, colors.HexColor("#F7FAFC")]),
                    ('GRID', (0,0), (-1,-1), 0.5, border_color),
                ]))
                
                story.append(KeepTogether([
                    Spacer(1, 6),
                    md_table,
                    Spacer(1, 6)
                ]))
                
        elif b_type == 'hr':
            story.append(Spacer(1, 8))
            story.append(Table([[ "" ]], colWidths=[504], rowHeights=[1], style=TableStyle([
                ('BACKGROUND', (0,0), (-1,-1), border_color),
                ('BOTTOMPADDING', (0,0), (-1,-1), 0),
                ('TOPPADDING', (0,0), (-1,-1), 0),
            ])))
            story.append(Spacer(1, 8))
            
        idx += 1
        
    # Signature block at the very end
    story.append(Spacer(1, 15))
    story.append(Table([[ "" ]], colWidths=[504], rowHeights=[1.5], style=TableStyle([
        ('BACKGROUND', (0,0), (-1,-1), primary_color),
        ('BOTTOMPADDING', (0,0), (-1,-1), 0),
        ('TOPPADDING', (0,0), (-1,-1), 0),
    ])))
    story.append(Spacer(1, 10))
    
    sig_text = (
        "<b>VERIFICATION SIGN OFF:</b><br/>"
        "This specification is verified for execution by coding copilots and agent runtimes. "
        "All parameters correspond to physical hardware EUI: <code>0x0016c001ff13ce58</code>.<br/>"
        "<i>Gateway Integrator:</i> astronautshe.com &bull; "
        "<i>Protocol Lead:</i> zymatica.space &bull; "
        "<i>Orchestrator Agent:</i> Devs One &bull; "
        "<i>Signed on behalf of:</i> TheAiCollective.art"
    )
    story.append(Paragraph(sig_text, body_style))
    
    print(f"Building PDF to: {pdf_path}")
    doc.build(story, canvasmaker=NumberedCanvas)
    print("[+] PDF built successfully.")

if __name__ == "__main__":
    base_dir = os.path.dirname(os.path.abspath(__file__))
    md_file = os.path.join(base_dir, "Zymatica_Voice_Lora_Guide.md")
    pdf_file = os.path.join(base_dir, "Zymatica_Voice_Lora_Guide.pdf")
    build_pdf(md_file, pdf_file)
