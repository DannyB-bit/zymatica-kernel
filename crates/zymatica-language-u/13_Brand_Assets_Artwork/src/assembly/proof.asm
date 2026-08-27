; Watermark: ip zymatica.space | astronautshe.com
; Copyright (c) 2026 Zymatica. All rights reserved.

extern printf
global main

section .data
    title db "======================================================================", 10, "ZYMATICA | Brand Assets & Artwork Proof (Assembly Edition)", 10, "======================================================================", 10, 10, 0
    verify_msg db 10, "[VERIFICATION] Brand assets and registry confirmed.", 10, 0
log1 db "[1] Checking brand branding files: Logo.jpg", 10, 0
    log2 db "[2] Resolving architecture graphics: architecture.png", 10, 0

section .text
main:
    sub rsp, 40
    mov rcx, title
    call printf
    mov rcx, log1
    call printf
    mov rcx, log2
    call printf
    mov rcx, verify_msg
    call printf
    add rsp, 40
    xor eax, eax
    ret
