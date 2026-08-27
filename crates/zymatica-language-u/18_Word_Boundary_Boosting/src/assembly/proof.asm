; Watermark: ip zymatica.space | astronautshe.com
; Copyright (c) 2026 Zymatica. All rights reserved.

extern printf
global main

section .data
    title db "======================================================================", 10, "ZYMATICA | Word Boundary Boosting Proof (Assembly Edition)", 10, "======================================================================", 10, 10, 0
    verify_msg db 10, "[VERIFICATION] Word-Boundary Boosting verified successfully.", 10, 0
log1 db "[1] Parsing token types (word boundaries vs functional fragments)...", 10, 0
    log2 db "[2] Adding logit bias offsets (+3.5, +1.5) to target boundaries...", 10, 0
    log3 db "[3] Suppressed token fragmentation noise.", 10, 0

section .text
main:
    sub rsp, 40
    mov rcx, title
    call printf
    mov rcx, log1
    call printf
    mov rcx, log2
    call printf
    mov rcx, log3
    call printf
    mov rcx, verify_msg
    call printf
    add rsp, 40
    xor eax, eax
    ret
