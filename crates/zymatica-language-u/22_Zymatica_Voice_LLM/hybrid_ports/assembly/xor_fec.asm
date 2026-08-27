; Watermark: ip zymatica.space | astronautshe.com
; Copyright (c) 2026 Zymatica. All rights reserved.

section .text
global xor_fec_byte_block
xor_fec_byte_block:
    ; rcx = ptr to packet A
    ; rdx = ptr to packet B (XOR parity)
    ; r8 = output ptr
    ; r9 = size in bytes
    xor rax, rax
.loop:
    cmp rax, r9
    jge .done
    mov r10b, [rcx + rax]
    xor r10b, [rdx + rax]
    mov [r8 + rax], r10b
    inc rax
    jmp .loop
.done:
    ret
