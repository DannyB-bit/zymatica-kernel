	.def	@feat.00;
	.scl	3;
	.type	0;
	.endef
	.globl	@feat.00
@feat.00 = 0
	.intel_syntax noprefix
	.file	"proof"
	.def	start.WinStartup;
	.scl	3;
	.type	32;
	.endef
	.text
	.p2align	4
start.WinStartup:
.Lfunc_begin0:
	.cv_func_id 0
	.cv_file	1 "C:\\Users\\freed\\AppData\\Local\\Microsoft\\WinGet\\Packages\\zig.zig_Microsoft.Winget.Source_8wekyb3d8bbwe\\zig-x86_64-windows-0.16.0\\lib\\std\\start.zig"
	.cv_loc	0 1 473 0
.seh_proc start.WinStartup
	push	rbp
	.seh_pushreg rbp
	sub	rsp, 32
	.seh_stackalloc 32
	lea	rbp, [rsp + 32]
	.seh_setframe rbp, 32
	.seh_endprologue
	and	rsp, -16
.Ltmp0:
	.cv_loc	0 1 475 45
	#APP
	fninit
	#NO_APP
	.cv_loc	0 1 484 44
	xor	ecx, ecx
	call	RtlExitUserProcess
	int3
.Ltmp1:
.Lfunc_end0:
	.seh_endproc

	.def	proof.run_verification;
	.scl	3;
	.type	32;
	.endef
	.p2align	4
proof.run_verification:
.Lfunc_begin1:
	.cv_func_id 1
	.cv_file	2 "J:\\Language-U\\WASM_U-Performance_Record\\proof.zig"
	.cv_loc	1 2 376 0
.seh_proc proof.run_verification
	push	rbp
	.seh_pushreg rbp
	push	rsi
	.seh_pushreg rsi
	push	rdi
	.seh_pushreg rdi
	mov	eax, 10320
	call	___chkstk_ms
	sub	rsp, rax
	.seh_stackalloc 10320
	lea	rbp, [rsp + 128]
	.seh_setframe rbp, 128
	.seh_endprologue
	xor	esi, esi
	lea	rdi, [rbp - 88]
.Ltmp2:
	.cv_inline_site_id 2 within 1 inlined_at 2 385 32
	.cv_loc	2 2 144 9
	mov	r8d, 10248
	mov	rcx, rdi
	xor	edx, edx
	call	memset
.Ltmp3:
	.cv_loc	1 2 386 11
	lea	rcx, [rip + __anon_5057]
	mov	edx, 5
	mov	r8, rdi
	call	proof.encode
	.cv_loc	1 2 388 34
	mov	rdx, qword ptr [rbp - 88]
	.cv_loc	1 2 388 45
	add	rdx, 7
	.cv_loc	1 2 388 50
	shr	rdx, 3
.Ltmp4:
	.cv_loc	1 2 390 18
	lea	rcx, [rbp - 80]
	lea	r9, [rbp + 10162]
	.cv_loc	1 2 390 11
	mov	r8d, 5
	call	proof.decode
.Ltmp5:
	.cv_loc	1 2 396 31
	cmp	byte ptr [rbp + 10162], 1
.Ltmp6:
	.cv_loc	1 2 397 34
	jne	.LBB1_15
.Ltmp7:
	cmp	byte ptr [rbp + 10163], 2
	jne	.LBB1_15
.Ltmp8:
	cmp	dword ptr [rbp + 10164], 100992003
	jne	.LBB1_15
.Ltmp9:
	.cv_loc	1 2 396 31
	cmp	byte ptr [rbp + 10168], 8
.Ltmp10:
	.cv_loc	1 2 397 34
	jne	.LBB1_15
.Ltmp11:
	cmp	byte ptr [rbp + 10169], 0
	jne	.LBB1_15
.Ltmp12:
	cmp	dword ptr [rbp + 10170], 251658511
	jne	.LBB1_15
.Ltmp13:
	.cv_loc	1 2 396 31
	cmp	byte ptr [rbp + 10174], 0
.Ltmp14:
	.cv_loc	1 2 397 34
	jne	.LBB1_15
.Ltmp15:
	cmp	byte ptr [rbp + 10175], 0
	jne	.LBB1_15
.Ltmp16:
	cmp	dword ptr [rbp + 10176], 0
	jne	.LBB1_15
.Ltmp17:
	.cv_loc	1 2 396 31
	cmp	byte ptr [rbp + 10180], 15
.Ltmp18:
	.cv_loc	1 2 397 34
	jne	.LBB1_15
.Ltmp19:
	cmp	byte ptr [rbp + 10181], 15
	jne	.LBB1_15
.Ltmp20:
	cmp	dword ptr [rbp + 10182], 252645135
	jne	.LBB1_15
.Ltmp21:
	.cv_loc	1 2 396 31
	cmp	byte ptr [rbp + 10186], 4
.Ltmp22:
	.cv_loc	1 2 397 34
	jne	.LBB1_15
.Ltmp23:
	cmp	byte ptr [rbp + 10187], 5
	jne	.LBB1_15
.Ltmp24:
	.cv_loc	1 2 398 34
	xor	esi, esi
	cmp	dword ptr [rbp + 10188], 151521030
	sete	sil
.Ltmp25:
.LBB1_15:
	mov	eax, esi
	.seh_startepilogue
	add	rsp, 10320
	pop	rdi
	pop	rsi
	pop	rbp
	.seh_endepilogue
	ret
.Ltmp26:
.Lfunc_end1:
	.seh_endproc

	.def	proof.decode;
	.scl	3;
	.type	32;
	.endef
	.p2align	4
proof.decode:
.Lfunc_begin2:
	.cv_func_id 3
	.cv_loc	3 2 269 0
.seh_proc proof.decode
	push	rbp
	.seh_pushreg rbp
	push	r15
	.seh_pushreg r15
	push	r14
	.seh_pushreg r14
	push	r13
	.seh_pushreg r13
	push	r12
	.seh_pushreg r12
	push	rsi
	.seh_pushreg rsi
	push	rdi
	.seh_pushreg rdi
	push	rbx
	.seh_pushreg rbx
	mov	eax, 10376
	call	___chkstk_ms
	sub	rsp, rax
	.seh_stackalloc 10376
	lea	rbp, [rsp + 128]
	.seh_setframe rbp, 128
	.seh_endprologue
	mov	qword ptr [rbp + 10224], r9
	mov	qword ptr [rbp + 10232], r8
.Ltmp27:
	mov	rbx, rdx
	mov	r14, rcx
.Ltmp28:
	.cv_inline_site_id 4 within 3 inlined_at 2 270 37
	.cv_loc	4 2 26 9
	vxorps	xmm0, xmm0, xmm0
	vmovaps	xmmword ptr [rbp - 80], xmm0
	mov	qword ptr [rbp - 64], 0
	movabs	rax, 549755813889
.Ltmp29:
	.cv_loc	3 2 270 37
	mov	qword ptr [rbp - 56], rax
	mov	word ptr [rbp + 9168], 0
	mov	byte ptr [rbp + 9170], 0
.Ltmp30:
	.cv_inline_site_id 5 within 3 inlined_at 2 271 27
	.cv_loc	5 2 181 38
	shl	rbx, 3
.Ltmp31:
	xor	r13d, r13d
	mov	rax, -32
.Ltmp32:
	xor	edi, edi
	jmp	.LBB2_1
.Ltmp33:
	.p2align	4
.LBB2_12:
	.cv_loc	3 2 276 50
	lea	r13d, [rcx + 4*r13]
.Ltmp34:
	or	r13d, edx
.Ltmp35:
	.cv_loc	3 2 275 12
	add	rax, 2
.Ltmp36:
	je	.LBB2_3
.Ltmp37:
.LBB2_1:
	.cv_inline_site_id 6 within 3 inlined_at 2 276 50
	.cv_loc	6 2 186 35
	cmp	rdi, rbx
	jae	.LBB2_2
.Ltmp38:
	.cv_loc	6 2 189 41
	mov	rcx, rdi
	shr	rcx, 3
.Ltmp39:
	.cv_loc	6 2 191 33
	movzx	ecx, byte ptr [r14 + rcx]
.Ltmp40:
	.cv_loc	6 2 191 44
	mov	edx, edi
	not	dl
	and	dl, 7
	shrx	ecx, ecx, edx
.Ltmp41:
	.cv_loc	6 2 192 24
	inc	rdi
.Ltmp42:
	.cv_loc	3 2 276 24
	add	ecx, ecx
.Ltmp43:
	and	ecx, 2
.Ltmp44:
	xor	edx, edx
.Ltmp45:
	.cv_loc	6 2 186 35
	cmp	rdi, rbx
	jae	.LBB2_12
	jmp	.LBB2_11
.Ltmp46:
	.p2align	4
.LBB2_2:
	xor	ecx, ecx
.Ltmp47:
	xor	edx, edx
.Ltmp48:
	cmp	rdi, rbx
	jae	.LBB2_12
.Ltmp49:
.LBB2_11:
	.cv_loc	6 2 189 41
	mov	rdx, rdi
	shr	rdx, 3
.Ltmp50:
	.cv_loc	6 2 191 33
	movzx	edx, byte ptr [r14 + rdx]
.Ltmp51:
	.cv_loc	6 2 191 44
	mov	r8d, edi
	not	r8b
	and	r8b, 7
	shrx	edx, edx, r8d
	and	edx, 1
.Ltmp52:
	.cv_loc	6 2 192 24
	inc	rdi
.Ltmp53:
	jmp	.LBB2_12
.Ltmp54:
.LBB2_3:
	.cv_loc	3 2 284 12
	cmp	qword ptr [rbp + 10232], 0
	je	.LBB2_15
.Ltmp55:
	mov	r15d, -1
	xor	esi, esi
	xor	r12d, r12d
	jmp	.LBB2_5
.Ltmp56:
	.p2align	4
.LBB2_14:
	.cv_loc	3 2 344 16
	mov	r12, qword ptr [rbp + 10216]
.Ltmp57:
	lea	rax, [r12 + 2*r12]
	.cv_loc	3 2 345 30
	movzx	edx, byte ptr [rbp + 10244]
	.cv_loc	3 2 345 34
	mov	ecx, edx
	shr	cl, 4
	mov	r10, qword ptr [rbp + 10224]
	mov	byte ptr [r10 + 2*rax], cl
	.cv_loc	3 2 346 33
	mov	ecx, edx
	and	cl, 15
	mov	byte ptr [r10 + 2*rax + 1], cl
	.cv_loc	3 2 347 33
	movzx	r8d, byte ptr [rbp + 10245]
	.cv_loc	3 2 347 37
	mov	ecx, r8d
	shr	cl, 4
	mov	byte ptr [r10 + 2*rax + 2], cl
	.cv_loc	3 2 348 32
	mov	ecx, r8d
	and	cl, 15
	mov	byte ptr [r10 + 2*rax + 3], cl
	.cv_loc	3 2 349 29
	movzx	r9d, byte ptr [rbp + 10246]
	.cv_loc	3 2 349 33
	mov	ecx, r9d
	shr	cl, 4
	mov	byte ptr [r10 + 2*rax + 4], cl
	.cv_loc	3 2 350 32
	mov	ecx, r9d
	and	cl, 15
	mov	byte ptr [r10 + 2*rax + 5], cl
	.cv_loc	3 2 352 21
	lea	rcx, [rbp - 80]
	call	proof.RadicalPredictor.observe
.Ltmp58:
	.cv_loc	3 2 284 43
	inc	r12
.Ltmp59:
	.cv_loc	3 2 284 12
	cmp	r12, qword ptr [rbp + 10232]
	je	.LBB2_15
.Ltmp60:
.LBB2_5:
	.cv_loc	3 2 285 29
	mov	qword ptr [rbp + 10216], r12
.Ltmp61:
	movzx	eax, byte ptr [rbp + 9168]
.Ltmp62:
	mov	byte ptr [rbp + 10241], al
.Ltmp63:
	.cv_loc	3 2 286 29
	movzx	eax, byte ptr [rbp + 9169]
.Ltmp64:
	mov	byte ptr [rbp + 10242], al
.Ltmp65:
	.cv_loc	3 2 287 29
	movzx	eax, byte ptr [rbp + 9170]
.Ltmp66:
	mov	byte ptr [rbp + 10243], al
.Ltmp67:
	.cv_loc	3 2 288 9
	mov	word ptr [rbp + 10244], 0
	mov	byte ptr [rbp + 10246], 0
.Ltmp68:
	xor	r12d, r12d
	jmp	.LBB2_6
.Ltmp69:
	.p2align	4
.LBB2_13:
	.cv_loc	3 2 291 34
	inc	r12
.Ltmp70:
	.cv_loc	3 2 291 16
	cmp	r12, 3
	je	.LBB2_14
.Ltmp71:
.LBB2_6:
	.cv_loc	3 2 292 21
	test	r12, r12
	je	.LBB2_16
.Ltmp72:
	cmp	r12, 1
	jne	.LBB2_26
.Ltmp73:
	.cv_loc	3 2 294 48
	movzx	edx, byte ptr [rbp + 10244]
	.cv_loc	3 2 294 40
	lea	rcx, [rbp - 80]
	movzx	r8d, byte ptr [rbp + 10242]
	lea	r9, [rbp + 9188]
	call	proof.RadicalPredictor.getCumFreqsRF
	jmp	.LBB2_17
.Ltmp74:
	.p2align	4
.LBB2_16:
	.cv_loc	3 2 293 40
	lea	rcx, [rbp - 80]
	movzx	edx, byte ptr [rbp + 10241]
	lea	r8, [rbp + 9188]
	call	proof.RadicalPredictor.getCumFreqsRC
	jmp	.LBB2_17
.Ltmp75:
	.p2align	4
.LBB2_26:
	.cv_loc	3 2 295 51
	movzx	edx, byte ptr [rbp + 10244]
	.cv_loc	3 2 295 63
	movzx	r8d, byte ptr [rbp + 10245]
	.cv_loc	3 2 295 43
	lea	rax, [rbp + 9188]
	mov	qword ptr [rsp + 32], rax
	lea	rcx, [rbp - 80]
	movzx	r9d, byte ptr [rbp + 10243]
	call	proof.RadicalPredictor.getCumFreqsRA
.Ltmp76:
.LBB2_17:
	.cv_loc	3 2 298 45
	mov	r8d, dword ptr [rbp + 10212]
.Ltmp77:
	.cv_loc	3 2 299 13
	mov	ecx, r15d
	mov	edx, esi
	.cv_loc	3 2 299 48
	sub	rcx, rdx
	.cv_loc	3 2 299 64
	inc	rcx
.Ltmp78:
	.cv_loc	3 2 300 13
	mov	eax, r13d
	.cv_loc	3 2 300 77
	sub	rax, rdx
	inc	rax
	.cv_loc	3 2 300 82
	imul	rax, r8
	.cv_loc	3 2 300 90
	dec	rax
	.cv_loc	3 2 300 32
	mov	rdx, rax
	or	rdx, rcx
	shr	rdx, 32
	test	rdx, rdx
	je	.LBB2_18
.Ltmp79:
	.cv_loc	3 2 300 32
	je	.LBB2_18
.Ltmp80:
	xor	edx, edx
	div	rcx
	jmp	.LBB2_21
.Ltmp81:
	.p2align	4
.LBB2_18:
	.cv_loc	3 2 300 32
	xor	edx, edx
	div	ecx
.Ltmp82:
.LBB2_21:
	xor	edx, edx
	mov	r9d, 255
	jmp	.LBB2_22
.Ltmp83:
	.p2align	4
.LBB2_41:
	.cv_loc	3 2 305 20
	lea	edx, [r10 + 1]
.Ltmp84:
	cmp	edx, r9d
	jg	.LBB2_43
.Ltmp85:
.LBB2_22:
	.cv_loc	3 2 306 41
	lea	r10d, [rdx + r9]
	.cv_loc	3 2 306 29
	shr	r10d
.Ltmp86:
	.cv_loc	3 2 307 39
	mov	r11d, dword ptr [rbp + 4*r10 + 9188]
	cmp	rax, r11
.Ltmp87:
	.cv_loc	3 2 307 39
	jb	.LBB2_24
.Ltmp88:
	.cv_loc	3 2 307 105
	mov	r11d, dword ptr [rbp + 4*r10 + 9192]
	cmp	rax, r11
	jb	.LBB2_27
.Ltmp89:
.LBB2_24:
	.cv_loc	3 2 310 60
	mov	r11d, dword ptr [rbp + 4*r10 + 9192]
	cmp	rax, r11
	jae	.LBB2_41
.Ltmp90:
	lea	r9d, [r10 - 1]
.Ltmp91:
	.cv_loc	3 2 305 20
	cmp	edx, r9d
	jle	.LBB2_22
.Ltmp92:
.LBB2_43:
	xor	r10d, r10d
.Ltmp93:
.LBB2_27:
	.cv_loc	3 2 317 20
	mov	byte ptr [rbp + r12 + 10244], r10b
.Ltmp94:
	.cv_loc	3 2 319 38
	movzx	eax, r10b
.Ltmp95:
	mov	r9d, dword ptr [rbp + 4*rax + 9188]
.Ltmp96:
	.cv_loc	3 2 320 39
	mov	eax, dword ptr [rbp + 4*rax + 9192]
.Ltmp97:
	.cv_loc	3 2 322 68
	imul	rax, rcx
	.cv_loc	3 2 322 46
	mov	rdx, rax
.Ltmp98:
	shr	rdx, 32
	test	rdx, rdx
	je	.LBB2_28
.Ltmp99:
	.cv_loc	3 2 322 46
	je	.LBB2_28
.Ltmp100:
	xor	edx, edx
	div	r8
	jmp	.LBB2_31
.Ltmp101:
	.p2align	4
.LBB2_28:
	.cv_loc	3 2 322 46
	xor	edx, edx
	div	r8d
.Ltmp102:
.LBB2_31:
	.cv_loc	3 2 322 36
	lea	r15d, [rsi + rax]
.Ltmp103:
	dec	r15d
.Ltmp104:
	.cv_loc	3 2 323 67
	imul	rcx, r9
.Ltmp105:
	.cv_loc	3 2 323 45
	mov	rax, rcx
	shr	rax, 32
	test	rax, rax
	jne	.LBB2_37
.Ltmp106:
	.cv_loc	3 2 323 45
	mov	eax, ecx
	xor	edx, edx
	div	r8d
	.cv_loc	3 2 323 35
	add	eax, esi
.Ltmp107:
	mov	esi, eax
.Ltmp108:
	.cv_loc	3 2 326 21
	test	r15d, r15d
	jns	.LBB2_34
	jmp	.LBB2_44
.Ltmp109:
	.p2align	4
.LBB2_37:
	.cv_loc	3 2 323 45
	je	.LBB2_38
.Ltmp110:
	mov	rax, rcx
	xor	edx, edx
	div	r8
	jmp	.LBB2_40
.Ltmp111:
.LBB2_38:
	mov	eax, ecx
	xor	edx, edx
	div	r8d
.Ltmp112:
.LBB2_40:
	.cv_loc	3 2 323 35
	add	eax, esi
.Ltmp113:
	mov	esi, eax
.Ltmp114:
	.cv_loc	3 2 326 21
	test	r15d, r15d
	jns	.LBB2_34
	jmp	.LBB2_44
.Ltmp115:
	.p2align	4
.LBB2_36:
	add	esi, esi
.Ltmp116:
	lea	r15d, [2*r15 + 1]
.Ltmp117:
	add	r13d, r13d
.Ltmp118:
	or	r13d, eax
.Ltmp119:
	test	r15d, r15d
	js	.LBB2_44
.Ltmp120:
.LBB2_34:
	.cv_inline_site_id 7 within 3 inlined_at 2 0 0
	.cv_loc	7 2 186 35
	xor	eax, eax
.Ltmp121:
	cmp	rdi, rbx
	jae	.LBB2_36
.Ltmp122:
	.cv_loc	7 2 189 41
	mov	rax, rdi
	shr	rax, 3
	.cv_loc	7 2 191 33
	movzx	eax, byte ptr [r14 + rax]
	.cv_loc	7 2 191 44
	mov	ecx, edi
	not	cl
	and	cl, 7
	shrx	eax, eax, ecx
	and	eax, 1
	.cv_loc	7 2 192 24
	inc	rdi
.Ltmp123:
	jmp	.LBB2_36
.Ltmp124:
	.p2align	4
.LBB2_44:
	.cv_loc	3 2 330 28
	test	esi, esi
	js	.LBB2_34
.Ltmp125:
	.cv_loc	3 2 334 28
	cmp	r15d, -1073741825
.Ltmp126:
	.cv_loc	3 2 334 28
	ja	.LBB2_13
.Ltmp127:
	cmp	esi, 1073741824
	jb	.LBB2_13
.Ltmp128:
	.cv_inline_site_id 8 within 3 inlined_at 2 337 77
	.cv_loc	8 2 186 35
	xor	eax, eax
.Ltmp129:
	cmp	rdi, rbx
	jae	.LBB2_49
.Ltmp130:
	.cv_loc	8 2 189 41
	mov	rax, rdi
	shr	rax, 3
.Ltmp131:
	.cv_loc	8 2 191 33
	movzx	eax, byte ptr [r14 + rax]
.Ltmp132:
	.cv_loc	8 2 191 44
	mov	ecx, edi
	not	cl
	and	cl, 7
	shrx	eax, eax, ecx
	and	eax, 1
.Ltmp133:
	.cv_loc	8 2 192 24
	inc	rdi
.Ltmp134:
.LBB2_49:
	.cv_loc	3 2 337 77
	lea	esi, [2*rsi - 2147483648]
.Ltmp135:
	add	r15d, r15d
	xor	r15d, -2147483647
.Ltmp136:
	add	r13d, r13d
.Ltmp137:
	add	r13d, eax
	add	r13d, -2147483648
.Ltmp138:
	.cv_loc	3 2 326 21
	test	r15d, r15d
	jns	.LBB2_34
	jmp	.LBB2_44
.Ltmp139:
.LBB2_15:
	.cv_loc	3 2 284 43
	.seh_startepilogue
	add	rsp, 10376
	pop	rbx
.Ltmp140:
	pop	rdi
.Ltmp141:
	pop	rsi
	pop	r12
	pop	r13
.Ltmp142:
	pop	r14
	pop	r15
	pop	rbp
	.seh_endepilogue
	ret
.Ltmp143:
.Lfunc_end2:
	.seh_endproc

	.def	proof.encode;
	.scl	3;
	.type	32;
	.endef
	.p2align	4
proof.encode:
.Lfunc_begin3:
	.cv_func_id 9
	.cv_loc	9 2 206 0
.seh_proc proof.encode
	push	rbp
	.seh_pushreg rbp
	push	r15
	.seh_pushreg r15
	push	r14
	.seh_pushreg r14
	push	r13
	.seh_pushreg r13
	push	r12
	.seh_pushreg r12
	push	rsi
	.seh_pushreg rsi
	push	rdi
	.seh_pushreg rdi
	push	rbx
	.seh_pushreg rbx
	mov	eax, 10376
	call	___chkstk_ms
	sub	rsp, rax
	.seh_stackalloc 10376
	lea	rbp, [rsp + 128]
	.seh_setframe rbp, 128
	.seh_endprologue
	mov	rsi, r8
	mov	qword ptr [rbp + 10232], rcx
.Ltmp144:
	.cv_inline_site_id 10 within 9 inlined_at 2 207 37
	.cv_loc	10 2 26 9
	vxorps	xmm0, xmm0, xmm0
	vmovaps	xmmword ptr [rbp - 80], xmm0
	mov	qword ptr [rbp - 64], 0
	movabs	rax, 549755813889
.Ltmp145:
	.cv_loc	9 2 207 37
	mov	qword ptr [rbp - 56], rax
	mov	word ptr [rbp + 9168], 0
	mov	byte ptr [rbp + 9170], 0
.Ltmp146:
	mov	qword ptr [rbp + 10224], rdx
.Ltmp147:
	.cv_loc	9 2 213 10
	test	rdx, rdx
	je	.LBB3_1
.Ltmp148:
	mov	r14d, -1
	xor	edi, edi
	xor	eax, eax
	xor	ebx, ebx
	jmp	.LBB3_7
.Ltmp149:
	.p2align	4
.LBB3_62:
	.cv_loc	9 2 258 21
	lea	rcx, [rbp - 80]
	mov	edx, r12d
	mov	r8d, r13d
	movzx	r9d, byte ptr [rbp + 10247]
	call	proof.RadicalPredictor.observe
	mov	rax, qword ptr [rbp + 10216]
.Ltmp150:
	.cv_loc	9 2 259 5
	inc	rax
.Ltmp151:
	.cv_loc	9 2 213 10
	cmp	rax, qword ptr [rbp + 10224]
	je	.LBB3_11
.Ltmp152:
.LBB3_7:
	.cv_loc	9 2 213 10
	mov	qword ptr [rbp + 10216], rax
.Ltmp153:
	lea	rax, [rax + 2*rax]
	mov	rcx, qword ptr [rbp + 10232]
	movzx	r12d, byte ptr [rcx + 2*rax]
.Ltmp154:
	movzx	r13d, byte ptr [rcx + 2*rax + 2]
.Ltmp155:
	movzx	edx, byte ptr [rcx + 2*rax + 4]
.Ltmp156:
	.cv_loc	9 2 214 30
	shl	r12b, 4
.Ltmp157:
	.cv_loc	9 2 214 39
	or	r12b, byte ptr [rcx + 2*rax + 1]
.Ltmp158:
	.cv_loc	9 2 215 33
	shl	r13b, 4
.Ltmp159:
	.cv_loc	9 2 215 42
	or	r13b, byte ptr [rcx + 2*rax + 3]
.Ltmp160:
	.cv_loc	9 2 216 29
	shl	dl, 4
.Ltmp161:
	.cv_loc	9 2 216 38
	or	dl, byte ptr [rcx + 2*rax + 5]
.Ltmp162:
	.cv_loc	9 2 217 9
	mov	byte ptr [rbp + 10241], r12b
	mov	byte ptr [rbp + 10242], r13b
	mov	byte ptr [rbp + 10247], dl
.Ltmp163:
	mov	byte ptr [rbp + 10243], dl
	.cv_loc	9 2 219 29
	movzx	eax, byte ptr [rbp + 9168]
.Ltmp164:
	mov	byte ptr [rbp + 10244], al
.Ltmp165:
	.cv_loc	9 2 220 29
	movzx	eax, byte ptr [rbp + 9169]
.Ltmp166:
	mov	byte ptr [rbp + 10245], al
.Ltmp167:
	.cv_loc	9 2 221 29
	movzx	eax, byte ptr [rbp + 9170]
.Ltmp168:
	mov	byte ptr [rbp + 10246], al
.Ltmp169:
	xor	r15d, r15d
	jmp	.LBB3_8
.Ltmp170:
	.p2align	4
.LBB3_61:
	.cv_loc	9 2 224 34
	inc	r15
.Ltmp171:
	.cv_loc	9 2 224 16
	cmp	r15, 3
	je	.LBB3_62
.Ltmp172:
.LBB3_8:
	.cv_loc	9 2 225 21
	test	r15, r15
	je	.LBB3_16
.Ltmp173:
	cmp	r15, 1
	jne	.LBB3_81
.Ltmp174:
	.cv_loc	9 2 227 40
	lea	rcx, [rbp - 80]
	mov	edx, r12d
	movzx	r8d, byte ptr [rbp + 10245]
	lea	r9, [rbp + 9188]
	call	proof.RadicalPredictor.getCumFreqsRF
	jmp	.LBB3_17
.Ltmp175:
	.p2align	4
.LBB3_16:
	.cv_loc	9 2 226 40
	lea	rcx, [rbp - 80]
	movzx	edx, byte ptr [rbp + 10244]
	lea	r8, [rbp + 9188]
	call	proof.RadicalPredictor.getCumFreqsRC
	jmp	.LBB3_17
.Ltmp176:
	.p2align	4
.LBB3_81:
	.cv_loc	9 2 228 43
	lea	rax, [rbp + 9188]
	mov	qword ptr [rsp + 32], rax
	lea	rcx, [rbp - 80]
	mov	edx, r12d
	mov	r8d, r13d
	movzx	r9d, byte ptr [rbp + 10246]
	call	proof.RadicalPredictor.getCumFreqsRA
.Ltmp177:
.LBB3_17:
	.cv_loc	9 2 231 43
	movzx	eax, byte ptr [rbp + r15 + 10241]
.Ltmp178:
	.cv_loc	9 2 232 36
	mov	r8d, dword ptr [rbp + 10212]
.Ltmp179:
	.cv_loc	9 2 233 38
	mov	r9d, dword ptr [rbp + 4*rax + 9188]
.Ltmp180:
	.cv_loc	9 2 234 39
	mov	eax, dword ptr [rbp + 4*rax + 9192]
.Ltmp181:
	.cv_loc	9 2 236 13
	mov	ecx, r14d
	mov	edx, edi
	.cv_loc	9 2 236 48
	sub	rcx, rdx
	.cv_loc	9 2 236 64
	inc	rcx
.Ltmp182:
	.cv_loc	9 2 237 68
	imul	rax, rcx
	.cv_loc	9 2 237 46
	mov	rdx, rax
	shr	rdx, 32
	test	rdx, rdx
	je	.LBB3_18
.Ltmp183:
	.cv_loc	9 2 237 46
	je	.LBB3_18
.Ltmp184:
	xor	edx, edx
	div	r8
	jmp	.LBB3_21
.Ltmp185:
	.p2align	4
.LBB3_18:
	.cv_loc	9 2 237 46
	xor	edx, edx
	div	r8d
.Ltmp186:
.LBB3_21:
	.cv_loc	9 2 237 36
	lea	r14d, [rdi + rax]
.Ltmp187:
	dec	r14d
.Ltmp188:
	.cv_loc	9 2 238 67
	imul	rcx, r9
.Ltmp189:
	.cv_loc	9 2 238 45
	mov	rax, rcx
	shr	rax, 32
	test	rax, rax
	jne	.LBB3_23
.Ltmp190:
	.cv_loc	9 2 238 45
	mov	eax, ecx
	xor	edx, edx
	div	r8d
	.cv_loc	9 2 238 35
	add	eax, edi
.Ltmp191:
	mov	edi, eax
.Ltmp192:
	.cv_loc	9 2 241 21
	test	r14d, r14d
	jns	.LBB3_28
	jmp	.LBB3_43
.Ltmp193:
	.p2align	4
.LBB3_23:
	.cv_loc	9 2 238 45
	je	.LBB3_24
.Ltmp194:
	mov	rax, rcx
	xor	edx, edx
	div	r8
	jmp	.LBB3_26
.Ltmp195:
.LBB3_24:
	mov	eax, ecx
	xor	edx, edx
	div	r8d
.Ltmp196:
.LBB3_26:
	.cv_loc	9 2 238 35
	add	eax, edi
.Ltmp197:
	mov	edi, eax
.Ltmp198:
	.cv_loc	9 2 241 21
	test	r14d, r14d
	jns	.LBB3_28
	jmp	.LBB3_43
.Ltmp199:
	.p2align	4
.LBB3_42:
	add	edi, edi
.Ltmp200:
	lea	r14d, [2*r14 + 1]
.Ltmp201:
	xor	ebx, ebx
	test	r14d, r14d
	js	.LBB3_43
.Ltmp202:
.LBB3_28:
	.cv_inline_site_id 11 within 9 inlined_at 2 242 42
	.cv_inline_site_id 12 within 11 inlined_at 2 164 22
	.cv_loc	12 2 151 30
	mov	rax, qword ptr [rsi]
.Ltmp203:
	.cv_loc	12 2 153 13
	cmp	rax, 81919
	ja	.LBB3_30
.Ltmp204:
	.cv_loc	12 2 151 41
	mov	rdx, rax
.Ltmp205:
	mov	ecx, eax
	not	cl
	.cv_loc	12 2 154 17
	mov	r8b, -2
	rol	r8b, cl
.Ltmp206:
	.cv_loc	12 2 151 41
	shr	rdx, 3
.Ltmp207:
	and	byte ptr [rsi + rdx + 8], r8b
.Ltmp208:
	.cv_loc	12 2 159 28
	inc	rax
.Ltmp209:
	mov	qword ptr [rsi], rax
.Ltmp210:
.LBB3_30:
	.cv_loc	11 2 165 16
	test	ebx, ebx
	je	.LBB3_42
.Ltmp211:
	.cv_inline_site_id 13 within 11 inlined_at 2 166 26
	.cv_loc	13 2 153 13
	test	bl, 1
	jne	.LBB3_33
.Ltmp212:
	.cv_loc	13 2 153 13
	mov	edx, ebx
	cmp	ebx, 1
	jne	.LBB3_37
	jmp	.LBB3_42
.Ltmp213:
	.p2align	4
.LBB3_33:
	.cv_loc	13 2 153 13
	cmp	rax, 81919
	ja	.LBB3_35
.Ltmp214:
	.cv_loc	13 2 152 33
	mov	ecx, eax
	not	cl
	.cv_loc	13 2 151 41
	mov	rdx, rax
.Ltmp215:
	and	cl, 7
	mov	r8b, 1
	shl	r8b, cl
.Ltmp216:
	shr	rdx, 3
.Ltmp217:
	or	byte ptr [rsi + rdx + 8], r8b
.Ltmp218:
	.cv_loc	13 2 159 28
	inc	rax
.Ltmp219:
	mov	qword ptr [rsi], rax
.Ltmp220:
.LBB3_35:
	.cv_loc	11 2 167 30
	lea	edx, [rbx - 1]
.Ltmp221:
	.cv_loc	13 2 153 13
	cmp	ebx, 1
	jne	.LBB3_37
	jmp	.LBB3_42
.Ltmp222:
	.p2align	4
.LBB3_40:
	mov	rcx, rax
.Ltmp223:
.LBB3_41:
	.cv_loc	11 2 165 16
	mov	rax, rcx
	add	edx, -2
	je	.LBB3_42
.Ltmp224:
.LBB3_37:
	.cv_loc	13 2 153 13
	cmp	rax, 81919
	ja	.LBB3_40
.Ltmp225:
	.cv_loc	13 2 152 33
	mov	ecx, eax
	not	cl
	.cv_loc	13 2 151 41
	mov	r8, rax
	shr	r8, 3
.Ltmp226:
	and	cl, 7
	mov	r9b, 1
	shl	r9b, cl
	or	byte ptr [rsi + r8 + 8], r9b
.Ltmp227:
	.cv_loc	13 2 159 28
	lea	r8, [rax + 1]
.Ltmp228:
	mov	qword ptr [rsi], r8
	mov	ecx, 81920
.Ltmp229:
	.cv_loc	13 2 153 13
	cmp	rax, 81919
	je	.LBB3_41
.Ltmp230:
	.cv_loc	13 2 152 33
	mov	ecx, r8d
	not	cl
.Ltmp231:
	and	cl, 7
	mov	r9b, 1
	shl	r9b, cl
.Ltmp232:
	.cv_loc	13 2 151 41
	shr	r8, 3
.Ltmp233:
	or	byte ptr [rsi + r8 + 8], r9b
.Ltmp234:
	.cv_loc	13 2 159 28
	add	rax, 2
	mov	qword ptr [rsi], rax
	jmp	.LBB3_40
.Ltmp235:
	.p2align	4
.LBB3_43:
	.cv_loc	9 2 245 28
	test	edi, edi
	js	.LBB3_44
.Ltmp236:
	.cv_loc	9 2 249 28
	cmp	r14d, -1073741825
.Ltmp237:
	.cv_loc	9 2 249 28
	ja	.LBB3_61
.Ltmp238:
	cmp	edi, 1073741824
	jb	.LBB3_61
.Ltmp239:
	.cv_loc	9 2 250 36
	inc	ebx
.Ltmp240:
	.cv_loc	9 2 251 46
	lea	edi, [2*rdi - 2147483648]
.Ltmp241:
	.cv_loc	9 2 252 49
	add	r14d, r14d
.Ltmp242:
	xor	r14d, -2147483647
.Ltmp243:
	.cv_loc	9 2 241 21
	test	r14d, r14d
	jns	.LBB3_28
	jmp	.LBB3_43
.Ltmp244:
	.p2align	4
.LBB3_44:
	.cv_inline_site_id 14 within 9 inlined_at 2 246 42
	.cv_inline_site_id 15 within 14 inlined_at 2 164 22
	.cv_loc	15 2 151 30
	mov	rax, qword ptr [rsi]
.Ltmp245:
	.cv_loc	15 2 153 13
	cmp	rax, 81919
	ja	.LBB3_46
.Ltmp246:
	.cv_loc	15 2 152 33
	mov	ecx, eax
	not	cl
	.cv_loc	15 2 151 41
	mov	rdx, rax
.Ltmp247:
	and	cl, 7
	mov	r8b, 1
	shl	r8b, cl
.Ltmp248:
	shr	rdx, 3
.Ltmp249:
	or	byte ptr [rsi + rdx + 8], r8b
.Ltmp250:
	.cv_loc	15 2 159 28
	inc	rax
.Ltmp251:
	mov	qword ptr [rsi], rax
.Ltmp252:
.LBB3_46:
	.cv_loc	14 2 165 16
	test	ebx, ebx
	je	.LBB3_42
.Ltmp253:
	.cv_inline_site_id 16 within 14 inlined_at 2 166 26
	.cv_loc	16 2 153 13
	test	bl, 1
	jne	.LBB3_49
.Ltmp254:
	.cv_loc	16 2 153 13
	mov	edx, ebx
	cmp	ebx, 1
	jne	.LBB3_53
	jmp	.LBB3_42
.Ltmp255:
.LBB3_49:
	.cv_loc	16 2 153 13
	cmp	rax, 81919
	ja	.LBB3_51
.Ltmp256:
	.cv_loc	16 2 151 41
	mov	rdx, rax
.Ltmp257:
	mov	ecx, eax
	not	cl
	.cv_loc	16 2 154 17
	mov	r8b, -2
	rol	r8b, cl
.Ltmp258:
	.cv_loc	16 2 151 41
	shr	rdx, 3
.Ltmp259:
	and	byte ptr [rsi + rdx + 8], r8b
.Ltmp260:
	.cv_loc	16 2 159 28
	inc	rax
.Ltmp261:
	mov	qword ptr [rsi], rax
.Ltmp262:
.LBB3_51:
	.cv_loc	14 2 167 30
	lea	edx, [rbx - 1]
.Ltmp263:
	.cv_loc	16 2 153 13
	cmp	ebx, 1
	jne	.LBB3_53
	jmp	.LBB3_42
.Ltmp264:
	.p2align	4
.LBB3_56:
	mov	r8, rax
.Ltmp265:
.LBB3_57:
	.cv_loc	14 2 165 16
	mov	rax, r8
	add	edx, -2
	je	.LBB3_42
.Ltmp266:
.LBB3_53:
	.cv_loc	16 2 153 13
	cmp	rax, 81919
	ja	.LBB3_56
.Ltmp267:
	.cv_loc	16 2 151 41
	mov	r8, rax
	shr	r8, 3
.Ltmp268:
	mov	ecx, eax
	not	cl
	.cv_loc	16 2 154 17
	mov	r9b, -2
	rol	r9b, cl
	and	byte ptr [rsi + r8 + 8], r9b
.Ltmp269:
	.cv_loc	16 2 159 28
	lea	rcx, [rax + 1]
	mov	qword ptr [rsi], rcx
	mov	r8d, 81920
.Ltmp270:
	.cv_loc	16 2 153 13
	cmp	rax, 81919
	je	.LBB3_57
.Ltmp271:
	.cv_loc	16 2 151 41
	mov	r8, rcx
.Ltmp272:
	not	cl
.Ltmp273:
	.cv_loc	16 2 154 17
	mov	r9b, -2
	rol	r9b, cl
.Ltmp274:
	.cv_loc	16 2 151 41
	shr	r8, 3
.Ltmp275:
	and	byte ptr [rsi + r8 + 8], r9b
.Ltmp276:
	.cv_loc	16 2 159 28
	add	rax, 2
	mov	qword ptr [rsi], rax
	jmp	.LBB3_56
.Ltmp277:
.LBB3_11:
	.cv_loc	9 2 261 20
	lea	edx, [rbx + 1]
.Ltmp278:
	.cv_inline_site_id 17 within 9 inlined_at 2 263 30
	.cv_inline_site_id 18 within 17 inlined_at 2 164 22
	.cv_loc	18 2 151 30
	mov	rax, qword ptr [rsi]
.Ltmp279:
	.cv_loc	9 2 262 9
	cmp	edi, 1073741824
	jb	.LBB3_2
.Ltmp280:
	.cv_inline_site_id 19 within 9 inlined_at 2 265 30
	.cv_inline_site_id 20 within 19 inlined_at 2 164 22
	.cv_loc	20 2 153 13
	cmp	rax, 81919
	ja	.LBB3_14
.Ltmp281:
	.cv_loc	20 2 152 33
	mov	ecx, eax
	not	cl
	.cv_loc	20 2 151 41
	mov	r8, rax
.Ltmp282:
	and	cl, 7
	mov	r9b, 1
	shl	r9b, cl
.Ltmp283:
	shr	r8, 3
.Ltmp284:
	or	byte ptr [rsi + r8 + 8], r9b
.Ltmp285:
	.cv_loc	20 2 159 28
	inc	rax
.Ltmp286:
	mov	qword ptr [rsi], rax
.Ltmp287:
.LBB3_14:
	test	bl, 1
	jne	.LBB3_15
.Ltmp288:
	.cv_inline_site_id 21 within 19 inlined_at 2 166 26
	.cv_loc	21 2 153 13
	cmp	rax, 81919
	ja	.LBB3_74
.Ltmp289:
	.cv_loc	21 2 151 41
	mov	rdx, rax
.Ltmp290:
	mov	ecx, eax
	not	cl
	mov	r8b, -2
	.cv_loc	21 2 154 17
	rol	r8b, cl
.Ltmp291:
	.cv_loc	21 2 151 41
	shr	rdx, 3
.Ltmp292:
	and	byte ptr [rsi + rdx + 8], r8b
.Ltmp293:
	.cv_loc	21 2 159 28
	inc	rax
.Ltmp294:
	mov	qword ptr [rsi], rax
.Ltmp295:
.LBB3_74:
	.cv_loc	19 2 165 16
	test	ebx, ebx
	jne	.LBB3_75
	jmp	.LBB3_80
.Ltmp296:
.LBB3_1:
	.cv_loc	18 2 151 30
	mov	rax, qword ptr [rsi]
	mov	edx, 1
.Ltmp297:
.LBB3_2:
	.cv_loc	18 2 153 13
	cmp	rax, 81919
	ja	.LBB3_4
.Ltmp298:
	.cv_loc	18 2 151 41
	mov	r8, rax
.Ltmp299:
	mov	ecx, eax
	not	cl
	mov	r9b, -2
	.cv_loc	18 2 154 17
	rol	r9b, cl
.Ltmp300:
	.cv_loc	18 2 151 41
	shr	r8, 3
.Ltmp301:
	and	byte ptr [rsi + r8 + 8], r9b
.Ltmp302:
	.cv_loc	18 2 159 28
	inc	rax
.Ltmp303:
	mov	qword ptr [rsi], rax
.Ltmp304:
.LBB3_4:
	test	dl, 1
	jne	.LBB3_63
.Ltmp305:
	.cv_loc	17 2 165 16
	mov	r8d, edx
	cmp	edx, 1
.Ltmp306:
	jne	.LBB3_67
	jmp	.LBB3_80
.Ltmp307:
.LBB3_63:
	.cv_inline_site_id 22 within 17 inlined_at 2 166 26
	.cv_loc	22 2 153 13
	cmp	rax, 81919
	ja	.LBB3_65
.Ltmp308:
	.cv_loc	22 2 152 33
	mov	ecx, eax
	not	cl
	.cv_loc	22 2 151 41
	mov	r8, rax
.Ltmp309:
	and	cl, 7
	mov	r9b, 1
	shl	r9b, cl
.Ltmp310:
	shr	r8, 3
.Ltmp311:
	or	byte ptr [rsi + r8 + 8], r9b
.Ltmp312:
	.cv_loc	22 2 159 28
	inc	rax
.Ltmp313:
	mov	qword ptr [rsi], rax
.Ltmp314:
.LBB3_65:
	.cv_loc	17 2 167 30
	lea	r8d, [rdx - 1]
	cmp	edx, 1
.Ltmp315:
	.cv_loc	17 2 165 16
	jne	.LBB3_67
.Ltmp316:
.LBB3_80:
	.cv_loc	9 2 265 30
	.seh_startepilogue
	add	rsp, 10376
	pop	rbx
	pop	rdi
	pop	rsi
.Ltmp317:
	pop	r12
	pop	r13
	pop	r14
	pop	r15
	pop	rbp
	.seh_endepilogue
	ret
.Ltmp318:
	.p2align	4
.LBB3_70:
	mov	rcx, rax
.Ltmp319:
.LBB3_71:
	.cv_loc	17 2 165 16
	mov	rax, rcx
.Ltmp320:
	add	r8d, -2
	je	.LBB3_80
.Ltmp321:
.LBB3_67:
	.cv_loc	22 2 153 13
	cmp	rax, 81919
	ja	.LBB3_70
.Ltmp322:
	.cv_loc	22 2 152 33
	mov	ecx, eax
	not	cl
	.cv_loc	22 2 151 41
	mov	rdx, rax
	shr	rdx, 3
.Ltmp323:
	and	cl, 7
	mov	r9b, 1
	shl	r9b, cl
	or	byte ptr [rsi + rdx + 8], r9b
.Ltmp324:
	.cv_loc	22 2 159 28
	lea	rdx, [rax + 1]
.Ltmp325:
	mov	qword ptr [rsi], rdx
	mov	ecx, 81920
.Ltmp326:
	.cv_loc	22 2 153 13
	cmp	rax, 81919
	je	.LBB3_71
.Ltmp327:
	.cv_loc	22 2 152 33
	mov	ecx, edx
	not	cl
.Ltmp328:
	and	cl, 7
	mov	r9b, 1
	shl	r9b, cl
.Ltmp329:
	.cv_loc	22 2 151 41
	shr	rdx, 3
.Ltmp330:
	or	byte ptr [rsi + rdx + 8], r9b
.Ltmp331:
	.cv_loc	22 2 159 28
	add	rax, 2
	mov	qword ptr [rsi], rax
	jmp	.LBB3_70
.Ltmp332:
.LBB3_15:
	mov	ebx, edx
	jmp	.LBB3_75
.Ltmp333:
	.p2align	4
.LBB3_78:
	mov	rdx, rax
.Ltmp334:
.LBB3_79:
	.cv_loc	19 2 165 16
	mov	rax, rdx
.Ltmp335:
	add	ebx, -2
	je	.LBB3_80
.Ltmp336:
.LBB3_75:
	.cv_loc	21 2 153 13
	cmp	rax, 81919
	ja	.LBB3_78
.Ltmp337:
	.cv_loc	21 2 151 41
	mov	rdx, rax
	shr	rdx, 3
.Ltmp338:
	mov	ecx, eax
	not	cl
	.cv_loc	21 2 154 17
	mov	r8b, -2
	rol	r8b, cl
	and	byte ptr [rsi + rdx + 8], r8b
.Ltmp339:
	.cv_loc	21 2 159 28
	lea	rcx, [rax + 1]
	mov	qword ptr [rsi], rcx
	mov	edx, 81920
.Ltmp340:
	.cv_loc	21 2 153 13
	cmp	rax, 81919
	je	.LBB3_79
.Ltmp341:
	.cv_loc	21 2 151 41
	mov	rdx, rcx
.Ltmp342:
	not	cl
.Ltmp343:
	.cv_loc	21 2 154 17
	mov	r8b, -2
	rol	r8b, cl
.Ltmp344:
	.cv_loc	21 2 151 41
	shr	rdx, 3
.Ltmp345:
	and	byte ptr [rsi + rdx + 8], r8b
.Ltmp346:
	.cv_loc	21 2 159 28
	add	rax, 2
	mov	qword ptr [rsi], rax
	jmp	.LBB3_78
.Ltmp347:
.Lfunc_end3:
	.seh_endproc

	.def	proof.RadicalPredictor.observe;
	.scl	3;
	.type	32;
	.endef
	.p2align	4
proof.RadicalPredictor.observe:
.Lfunc_begin4:
	.cv_func_id 23
	.cv_loc	23 2 41 0
.seh_proc proof.RadicalPredictor.observe
	push	rbp
	.seh_pushreg rbp
.Ltmp348:
	push	rsi
	.seh_pushreg rsi
	push	rdi
	.seh_pushreg rdi
	push	rbx
	.seh_pushreg rbx
	mov	rbp, rsp
	.seh_setframe rbp, 0
	.seh_endprologue
	.cv_loc	23 2 42 23
	mov	eax, dword ptr [rcx + 28]
.Ltmp349:
	.cv_loc	23 2 45 37
	movzx	r11d, byte ptr [rcx + 9248]
.Ltmp350:
	.cv_loc	23 2 47 35
	mov	r10, qword ptr [rcx]
.Ltmp351:
	.cv_loc	23 2 47 27
	test	r10, r10
	je	.LBB4_18
.Ltmp352:
	.cv_loc	23 2 48 22
	lea	rsi, [rcx + 40]
	mov	rdi, r10
	jmp	.LBB4_2
.Ltmp353:
	.p2align	4
.LBB4_16:
	.cv_loc	23 2 47 27
	add	rsi, 12
	dec	rdi
	je	.LBB4_17
.Ltmp354:
.LBB4_2:
	.cv_loc	23 2 48 22
	cmp	dword ptr [rsi - 8], r11d
.Ltmp355:
	.cv_loc	23 2 48 22
	jne	.LBB4_16
.Ltmp356:
	.cv_loc	23 2 48 46
	cmp	byte ptr [rsi], dl
.Ltmp357:
	.cv_loc	23 2 48 46
	jne	.LBB4_16
.Ltmp358:
	.cv_loc	23 2 49 29
	add	dword ptr [rsi - 4], eax
.Ltmp359:
	jmp	.LBB4_5
.Ltmp360:
.LBB4_17:
	.cv_loc	23 2 54 31
	cmp	r10, 255
.Ltmp361:
	.cv_loc	23 2 54 31
	ja	.LBB4_5
.Ltmp362:
.LBB4_18:
	.cv_loc	23 2 55 26
	lea	rsi, [rcx + 32]
.Ltmp363:
	lea	rdi, [r10 + 2*r10]
	mov	dword ptr [rsi + 4*rdi], r11d
	mov	byte ptr [rsi + 4*rdi + 8], dl
	mov	dword ptr [rsi + 4*rdi + 4], eax
	.cv_loc	23 2 56 31
	inc	r10
	mov	qword ptr [rcx], r10
.Ltmp364:
.LBB4_5:
	.cv_loc	23 2 60 9
	movzx	r10d, dl
	.cv_loc	23 2 60 38
	mov	r11d, r10d
.Ltmp365:
	shl	r11d, 8
	.cv_loc	23 2 60 59
	movzx	esi, byte ptr [rcx + 9249]
	or	esi, r11d
.Ltmp366:
	.cv_loc	23 2 62 35
	mov	r11, qword ptr [rcx + 8]
.Ltmp367:
	.cv_loc	23 2 62 27
	test	r11, r11
	je	.LBB4_21
.Ltmp368:
	.cv_loc	23 2 63 22
	lea	rdi, [rcx + 3112]
	mov	rbx, r11
	jmp	.LBB4_7
.Ltmp369:
	.p2align	4
.LBB4_19:
	.cv_loc	23 2 62 27
	add	rdi, 12
	dec	rbx
	je	.LBB4_20
.Ltmp370:
.LBB4_7:
	.cv_loc	23 2 63 22
	cmp	dword ptr [rdi - 8], esi
.Ltmp371:
	.cv_loc	23 2 63 22
	jne	.LBB4_19
.Ltmp372:
	.cv_loc	23 2 63 46
	cmp	byte ptr [rdi], r8b
.Ltmp373:
	.cv_loc	23 2 63 46
	jne	.LBB4_19
.Ltmp374:
	.cv_loc	23 2 64 29
	add	dword ptr [rdi - 4], eax
.Ltmp375:
	jmp	.LBB4_10
.Ltmp376:
.LBB4_20:
	.cv_loc	23 2 69 31
	cmp	r11, 255
.Ltmp377:
	.cv_loc	23 2 69 31
	ja	.LBB4_10
.Ltmp378:
.LBB4_21:
	.cv_loc	23 2 70 26
	lea	rdi, [rcx + 3104]
.Ltmp379:
	lea	rbx, [r11 + 2*r11]
	mov	dword ptr [rdi + 4*rbx], esi
	mov	byte ptr [rdi + 4*rbx + 8], r8b
	mov	dword ptr [rdi + 4*rbx + 4], eax
	.cv_loc	23 2 71 31
	inc	r11
	mov	qword ptr [rcx + 8], r11
.Ltmp380:
.LBB4_10:
	.cv_loc	23 2 75 38
	shl	r10d, 16
	movzx	esi, r8b
.Ltmp381:
	.cv_loc	23 2 75 61
	shl	esi, 8
	or	esi, r10d
	.cv_loc	23 2 75 82
	movzx	r11d, byte ptr [rcx + 9250]
	or	r11d, esi
.Ltmp382:
	.cv_loc	23 2 77 35
	mov	r10, qword ptr [rcx + 16]
.Ltmp383:
	.cv_loc	23 2 77 27
	test	r10, r10
	je	.LBB4_24
.Ltmp384:
	.cv_loc	23 2 78 22
	lea	rsi, [rcx + 6184]
	mov	rdi, r10
	jmp	.LBB4_12
.Ltmp385:
	.p2align	4
.LBB4_22:
	.cv_loc	23 2 77 27
	add	rsi, 12
	dec	rdi
	je	.LBB4_23
.Ltmp386:
.LBB4_12:
	.cv_loc	23 2 78 22
	cmp	dword ptr [rsi - 8], r11d
.Ltmp387:
	.cv_loc	23 2 78 22
	jne	.LBB4_22
.Ltmp388:
	.cv_loc	23 2 78 46
	cmp	byte ptr [rsi], r9b
.Ltmp389:
	.cv_loc	23 2 78 46
	jne	.LBB4_22
.Ltmp390:
	.cv_loc	23 2 79 29
	add	dword ptr [rsi - 4], eax
.Ltmp391:
	jmp	.LBB4_15
.Ltmp392:
.LBB4_23:
	.cv_loc	23 2 84 31
	cmp	r10, 255
.Ltmp393:
	.cv_loc	23 2 84 31
	ja	.LBB4_15
.Ltmp394:
.LBB4_24:
	.cv_loc	23 2 85 26
	lea	rsi, [rcx + 6176]
.Ltmp395:
	lea	rdi, [r10 + 2*r10]
	mov	dword ptr [rsi + 4*rdi], r11d
	mov	byte ptr [rsi + 4*rdi + 8], r9b
	mov	dword ptr [rsi + 4*rdi + 4], eax
	.cv_loc	23 2 86 31
	inc	r10
	mov	qword ptr [rcx + 16], r10
.Ltmp396:
.LBB4_15:
	.cv_loc	23 2 89 13
	mov	byte ptr [rcx + 9248], dl
	.cv_loc	23 2 90 13
	mov	byte ptr [rcx + 9249], r8b
	.cv_loc	23 2 91 13
	mov	byte ptr [rcx + 9250], r9b
	.seh_startepilogue
	pop	rbx
	pop	rdi
	pop	rsi
	pop	rbp
	.seh_endepilogue
	ret
.Ltmp397:
.Lfunc_end4:
	.seh_endproc

	.def	proof.RadicalPredictor.getCumFreqsRA;
	.scl	3;
	.type	32;
	.endef
	.p2align	4
proof.RadicalPredictor.getCumFreqsRA:
.Lfunc_begin5:
	.cv_func_id 24
	.cv_loc	24 2 123 0
.seh_proc proof.RadicalPredictor.getCumFreqsRA
	push	rbp
	.seh_pushreg rbp
	push	rsi
	.seh_pushreg rsi
	push	rdi
	.seh_pushreg rdi
	sub	rsp, 1024
	.seh_stackalloc 1024
	lea	rbp, [rsp + 128]
	.seh_setframe rbp, 128
	.seh_endprologue
.Ltmp398:
	mov	rax, qword ptr [rbp + 960]
.Ltmp399:
	.cv_loc	24 2 124 32
	vbroadcastss	ymm0, dword ptr [rcx + 24]
	vmovups	ymmword ptr [rbp - 128], ymm0
	vmovups	ymmword ptr [rbp - 96], ymm0
	vmovups	ymmword ptr [rbp - 64], ymm0
	vmovups	ymmword ptr [rbp - 32], ymm0
	vmovups	ymmword ptr [rbp], ymm0
	vmovups	ymmword ptr [rbp + 32], ymm0
	vmovups	ymmword ptr [rbp + 64], ymm0
	vmovups	ymmword ptr [rbp + 96], ymm0
	vmovups	ymmword ptr [rbp + 128], ymm0
	vmovups	ymmword ptr [rbp + 160], ymm0
	vmovups	ymmword ptr [rbp + 192], ymm0
	vmovups	ymmword ptr [rbp + 224], ymm0
	vmovups	ymmword ptr [rbp + 256], ymm0
	vmovups	ymmword ptr [rbp + 288], ymm0
	vmovups	ymmword ptr [rbp + 320], ymm0
	vmovups	ymmword ptr [rbp + 352], ymm0
	vmovups	ymmword ptr [rbp + 384], ymm0
	vmovups	ymmword ptr [rbp + 416], ymm0
	vmovups	ymmword ptr [rbp + 448], ymm0
	vmovups	ymmword ptr [rbp + 480], ymm0
	vmovups	ymmword ptr [rbp + 512], ymm0
	vmovups	ymmword ptr [rbp + 544], ymm0
	vmovups	ymmword ptr [rbp + 576], ymm0
	vmovups	ymmword ptr [rbp + 608], ymm0
	vmovups	ymmword ptr [rbp + 640], ymm0
	vmovups	ymmword ptr [rbp + 672], ymm0
	vmovups	ymmword ptr [rbp + 704], ymm0
	vmovups	ymmword ptr [rbp + 736], ymm0
	vmovups	ymmword ptr [rbp + 768], ymm0
	vmovups	ymmword ptr [rbp + 800], ymm0
	vmovups	ymmword ptr [rbp + 832], ymm0
	vmovups	ymmword ptr [rbp + 864], ymm0
.Ltmp400:
	.cv_loc	24 2 126 35
	mov	r10, qword ptr [rcx + 16]
.Ltmp401:
	.cv_loc	24 2 126 27
	test	r10, r10
	je	.LBB5_18
.Ltmp402:
	.cv_loc	24 2 127 22
	movzx	edx, dl
.Ltmp403:
	shl	edx, 16
	movzx	r8d, r8b
.Ltmp404:
	shl	r8d, 8
	or	r8d, edx
	movzx	edx, r9b
	or	edx, r8d
.Ltmp405:
	mov	r8d, r10d
	and	r8d, 3
	cmp	r10, 4
	jae	.LBB5_3
.Ltmp406:
	.cv_loc	24 2 127 22
	xor	r9d, r9d
.Ltmp407:
	test	r8, r8
	jne	.LBB5_14
	jmp	.LBB5_18
.Ltmp408:
.LBB5_3:
	.cv_loc	24 2 127 22
	and	r10, -4
	lea	r9, [rcx + 6220]
.Ltmp409:
	mov	r11, r10
	jmp	.LBB5_4
.Ltmp410:
	.p2align	4
.LBB5_11:
	.cv_loc	24 2 126 27
	add	r9, 48
	add	r11, -4
	je	.LBB5_12
.Ltmp411:
.LBB5_4:
	.cv_loc	24 2 127 22
	cmp	dword ptr [r9 - 44], edx
	je	.LBB5_21
.Ltmp412:
	.cv_loc	24 2 127 22
	cmp	dword ptr [r9 - 32], edx
	je	.LBB5_6
.Ltmp413:
.LBB5_7:
	.cv_loc	24 2 127 22
	cmp	dword ptr [r9 - 20], edx
	je	.LBB5_8
.Ltmp414:
.LBB5_9:
	.cv_loc	24 2 127 22
	cmp	dword ptr [r9 - 8], edx
	jne	.LBB5_11
	jmp	.LBB5_10
.Ltmp415:
	.p2align	4
.LBB5_21:
	.cv_loc	24 2 126 27
	movzx	esi, byte ptr [r9 - 36]
.Ltmp416:
	mov	edi, dword ptr [r9 - 40]
.Ltmp417:
	.cv_loc	24 2 128 34
	add	dword ptr [rbp + 4*rsi - 128], edi
.Ltmp418:
	.cv_loc	24 2 127 22
	cmp	dword ptr [r9 - 32], edx
	jne	.LBB5_7
.Ltmp419:
.LBB5_6:
	.cv_loc	24 2 126 27
	movzx	esi, byte ptr [r9 - 24]
.Ltmp420:
	mov	edi, dword ptr [r9 - 28]
.Ltmp421:
	.cv_loc	24 2 128 34
	add	dword ptr [rbp + 4*rsi - 128], edi
.Ltmp422:
	.cv_loc	24 2 127 22
	cmp	dword ptr [r9 - 20], edx
	jne	.LBB5_9
.Ltmp423:
.LBB5_8:
	.cv_loc	24 2 126 27
	movzx	esi, byte ptr [r9 - 12]
.Ltmp424:
	mov	edi, dword ptr [r9 - 16]
.Ltmp425:
	.cv_loc	24 2 128 34
	add	dword ptr [rbp + 4*rsi - 128], edi
.Ltmp426:
	.cv_loc	24 2 127 22
	cmp	dword ptr [r9 - 8], edx
	jne	.LBB5_11
.Ltmp427:
.LBB5_10:
	.cv_loc	24 2 126 27
	movzx	esi, byte ptr [r9]
.Ltmp428:
	mov	edi, dword ptr [r9 - 4]
.Ltmp429:
	.cv_loc	24 2 128 34
	add	dword ptr [rbp + 4*rsi - 128], edi
	jmp	.LBB5_11
.Ltmp430:
.LBB5_12:
	.cv_loc	24 2 127 22
	shl	r10, 2
	lea	r9, [r10 + 2*r10]
	.cv_loc	24 2 127 22
	test	r8, r8
	je	.LBB5_18
.Ltmp431:
.LBB5_14:
	.cv_loc	24 2 127 22
	add	rcx, r9
.Ltmp432:
	add	rcx, 6184
	shl	r8d, 2
	lea	r8, [r8 + 2*r8]
	xor	r9d, r9d
	jmp	.LBB5_15
.Ltmp433:
	.p2align	4
.LBB5_17:
	.cv_loc	24 2 126 27
	add	r9, 12
	cmp	r8, r9
	je	.LBB5_18
.Ltmp434:
.LBB5_15:
	.cv_loc	24 2 127 22
	cmp	dword ptr [rcx + r9 - 8], edx
	jne	.LBB5_17
.Ltmp435:
	.cv_loc	24 2 126 27
	movzx	r10d, byte ptr [rcx + r9]
.Ltmp436:
	mov	r11d, dword ptr [rcx + r9 - 4]
.Ltmp437:
	.cv_loc	24 2 128 34
	add	dword ptr [rbp + 4*r10 - 128], r11d
	jmp	.LBB5_17
.Ltmp438:
.LBB5_18:
	.cv_loc	24 2 131 18
	mov	dword ptr [rax], 0
	xor	ecx, ecx
.Ltmp439:
	xor	edx, edx
.Ltmp440:
	.p2align	4
.LBB5_19:
	.cv_loc	24 2 134 45
	add	ecx, dword ptr [rbp + 4*rdx - 128]
	mov	dword ptr [rax + 4*rdx + 4], ecx
.Ltmp441:
	add	ecx, dword ptr [rbp + 4*rdx - 124]
	mov	dword ptr [rax + 4*rdx + 8], ecx
.Ltmp442:
	add	ecx, dword ptr [rbp + 4*rdx - 120]
	mov	dword ptr [rax + 4*rdx + 12], ecx
.Ltmp443:
	add	ecx, dword ptr [rbp + 4*rdx - 116]
	mov	dword ptr [rax + 4*rdx + 16], ecx
.Ltmp444:
	add	ecx, dword ptr [rbp + 4*rdx - 112]
	mov	dword ptr [rax + 4*rdx + 20], ecx
.Ltmp445:
	add	ecx, dword ptr [rbp + 4*rdx - 108]
	mov	dword ptr [rax + 4*rdx + 24], ecx
.Ltmp446:
	add	ecx, dword ptr [rbp + 4*rdx - 104]
	mov	dword ptr [rax + 4*rdx + 28], ecx
.Ltmp447:
	add	ecx, dword ptr [rbp + 4*rdx - 100]
	mov	dword ptr [rax + 4*rdx + 32], ecx
.Ltmp448:
	.cv_loc	24 2 134 25
	add	rdx, 8
.Ltmp449:
	.cv_loc	24 2 133 16
	cmp	rdx, 256
	jne	.LBB5_19
.Ltmp450:
	.cv_loc	24 2 133 30
	.seh_startepilogue
	add	rsp, 1024
	pop	rdi
	pop	rsi
	pop	rbp
	.seh_endepilogue
	vzeroupper
	ret
.Ltmp451:
.Lfunc_end5:
	.seh_endproc

	.def	proof.RadicalPredictor.getCumFreqsRF;
	.scl	3;
	.type	32;
	.endef
	.p2align	4
proof.RadicalPredictor.getCumFreqsRF:
.Lfunc_begin6:
	.cv_func_id 25
	.cv_loc	25 2 108 0
.seh_proc proof.RadicalPredictor.getCumFreqsRF
	push	rbp
	.seh_pushreg rbp
.Ltmp452:
	push	rsi
	.seh_pushreg rsi
	push	rdi
	.seh_pushreg rdi
	sub	rsp, 1024
	.seh_stackalloc 1024
	lea	rbp, [rsp + 128]
	.seh_setframe rbp, 128
	.seh_endprologue
	.cv_loc	25 2 109 32
	vbroadcastss	ymm0, dword ptr [rcx + 24]
	vmovups	ymmword ptr [rbp - 128], ymm0
	vmovups	ymmword ptr [rbp - 96], ymm0
	vmovups	ymmword ptr [rbp - 64], ymm0
	vmovups	ymmword ptr [rbp - 32], ymm0
	vmovups	ymmword ptr [rbp], ymm0
	vmovups	ymmword ptr [rbp + 32], ymm0
	vmovups	ymmword ptr [rbp + 64], ymm0
	vmovups	ymmword ptr [rbp + 96], ymm0
	vmovups	ymmword ptr [rbp + 128], ymm0
	vmovups	ymmword ptr [rbp + 160], ymm0
	vmovups	ymmword ptr [rbp + 192], ymm0
	vmovups	ymmword ptr [rbp + 224], ymm0
	vmovups	ymmword ptr [rbp + 256], ymm0
	vmovups	ymmword ptr [rbp + 288], ymm0
	vmovups	ymmword ptr [rbp + 320], ymm0
	vmovups	ymmword ptr [rbp + 352], ymm0
	vmovups	ymmword ptr [rbp + 384], ymm0
	vmovups	ymmword ptr [rbp + 416], ymm0
	vmovups	ymmword ptr [rbp + 448], ymm0
	vmovups	ymmword ptr [rbp + 480], ymm0
	vmovups	ymmword ptr [rbp + 512], ymm0
	vmovups	ymmword ptr [rbp + 544], ymm0
	vmovups	ymmword ptr [rbp + 576], ymm0
	vmovups	ymmword ptr [rbp + 608], ymm0
	vmovups	ymmword ptr [rbp + 640], ymm0
	vmovups	ymmword ptr [rbp + 672], ymm0
	vmovups	ymmword ptr [rbp + 704], ymm0
	vmovups	ymmword ptr [rbp + 736], ymm0
	vmovups	ymmword ptr [rbp + 768], ymm0
	vmovups	ymmword ptr [rbp + 800], ymm0
	vmovups	ymmword ptr [rbp + 832], ymm0
	vmovups	ymmword ptr [rbp + 864], ymm0
.Ltmp453:
	.cv_loc	25 2 111 35
	mov	r10, qword ptr [rcx + 8]
.Ltmp454:
	.cv_loc	25 2 111 27
	test	r10, r10
	je	.LBB6_18
.Ltmp455:
	.cv_loc	25 2 112 22
	movzx	edx, dl
.Ltmp456:
	shl	edx, 8
	movzx	eax, r8b
	or	eax, edx
.Ltmp457:
	mov	edx, r10d
	and	edx, 3
	cmp	r10, 4
	jae	.LBB6_3
.Ltmp458:
	.cv_loc	25 2 112 22
	xor	r8d, r8d
.Ltmp459:
	test	rdx, rdx
	jne	.LBB6_14
	jmp	.LBB6_18
.Ltmp460:
.LBB6_3:
	.cv_loc	25 2 112 22
	and	r10, -4
	lea	r8, [rcx + 3148]
.Ltmp461:
	mov	r11, r10
	jmp	.LBB6_4
.Ltmp462:
	.p2align	4
.LBB6_11:
	.cv_loc	25 2 111 27
	add	r8, 48
	add	r11, -4
	je	.LBB6_12
.Ltmp463:
.LBB6_4:
	.cv_loc	25 2 112 22
	cmp	dword ptr [r8 - 44], eax
	je	.LBB6_21
.Ltmp464:
	.cv_loc	25 2 112 22
	cmp	dword ptr [r8 - 32], eax
	je	.LBB6_6
.Ltmp465:
.LBB6_7:
	.cv_loc	25 2 112 22
	cmp	dword ptr [r8 - 20], eax
	je	.LBB6_8
.Ltmp466:
.LBB6_9:
	.cv_loc	25 2 112 22
	cmp	dword ptr [r8 - 8], eax
	jne	.LBB6_11
	jmp	.LBB6_10
.Ltmp467:
	.p2align	4
.LBB6_21:
	.cv_loc	25 2 111 27
	movzx	esi, byte ptr [r8 - 36]
.Ltmp468:
	mov	edi, dword ptr [r8 - 40]
.Ltmp469:
	.cv_loc	25 2 113 34
	add	dword ptr [rbp + 4*rsi - 128], edi
.Ltmp470:
	.cv_loc	25 2 112 22
	cmp	dword ptr [r8 - 32], eax
	jne	.LBB6_7
.Ltmp471:
.LBB6_6:
	.cv_loc	25 2 111 27
	movzx	esi, byte ptr [r8 - 24]
.Ltmp472:
	mov	edi, dword ptr [r8 - 28]
.Ltmp473:
	.cv_loc	25 2 113 34
	add	dword ptr [rbp + 4*rsi - 128], edi
.Ltmp474:
	.cv_loc	25 2 112 22
	cmp	dword ptr [r8 - 20], eax
	jne	.LBB6_9
.Ltmp475:
.LBB6_8:
	.cv_loc	25 2 111 27
	movzx	esi, byte ptr [r8 - 12]
.Ltmp476:
	mov	edi, dword ptr [r8 - 16]
.Ltmp477:
	.cv_loc	25 2 113 34
	add	dword ptr [rbp + 4*rsi - 128], edi
.Ltmp478:
	.cv_loc	25 2 112 22
	cmp	dword ptr [r8 - 8], eax
	jne	.LBB6_11
.Ltmp479:
.LBB6_10:
	.cv_loc	25 2 111 27
	movzx	esi, byte ptr [r8]
.Ltmp480:
	mov	edi, dword ptr [r8 - 4]
.Ltmp481:
	.cv_loc	25 2 113 34
	add	dword ptr [rbp + 4*rsi - 128], edi
	jmp	.LBB6_11
.Ltmp482:
.LBB6_12:
	.cv_loc	25 2 112 22
	shl	r10, 2
	lea	r8, [r10 + 2*r10]
	.cv_loc	25 2 112 22
	test	rdx, rdx
	je	.LBB6_18
.Ltmp483:
.LBB6_14:
	.cv_loc	25 2 112 22
	add	rcx, r8
.Ltmp484:
	add	rcx, 3112
	shl	edx, 2
	lea	rdx, [rdx + 2*rdx]
	xor	r8d, r8d
	jmp	.LBB6_15
.Ltmp485:
	.p2align	4
.LBB6_17:
	.cv_loc	25 2 111 27
	add	r8, 12
	cmp	rdx, r8
	je	.LBB6_18
.Ltmp486:
.LBB6_15:
	.cv_loc	25 2 112 22
	cmp	dword ptr [rcx + r8 - 8], eax
	jne	.LBB6_17
.Ltmp487:
	.cv_loc	25 2 111 27
	movzx	r10d, byte ptr [rcx + r8]
.Ltmp488:
	mov	r11d, dword ptr [rcx + r8 - 4]
.Ltmp489:
	.cv_loc	25 2 113 34
	add	dword ptr [rbp + 4*r10 - 128], r11d
	jmp	.LBB6_17
.Ltmp490:
.LBB6_18:
	.cv_loc	25 2 116 18
	mov	dword ptr [r9], 0
	xor	eax, eax
.Ltmp491:
	xor	ecx, ecx
.Ltmp492:
	.p2align	4
.LBB6_19:
	.cv_loc	25 2 119 45
	add	eax, dword ptr [rbp + 4*rcx - 128]
	mov	dword ptr [r9 + 4*rcx + 4], eax
.Ltmp493:
	add	eax, dword ptr [rbp + 4*rcx - 124]
	mov	dword ptr [r9 + 4*rcx + 8], eax
.Ltmp494:
	add	eax, dword ptr [rbp + 4*rcx - 120]
	mov	dword ptr [r9 + 4*rcx + 12], eax
.Ltmp495:
	add	eax, dword ptr [rbp + 4*rcx - 116]
	mov	dword ptr [r9 + 4*rcx + 16], eax
.Ltmp496:
	add	eax, dword ptr [rbp + 4*rcx - 112]
	mov	dword ptr [r9 + 4*rcx + 20], eax
.Ltmp497:
	add	eax, dword ptr [rbp + 4*rcx - 108]
	mov	dword ptr [r9 + 4*rcx + 24], eax
.Ltmp498:
	add	eax, dword ptr [rbp + 4*rcx - 104]
	mov	dword ptr [r9 + 4*rcx + 28], eax
.Ltmp499:
	add	eax, dword ptr [rbp + 4*rcx - 100]
	mov	dword ptr [r9 + 4*rcx + 32], eax
.Ltmp500:
	.cv_loc	25 2 119 25
	add	rcx, 8
.Ltmp501:
	.cv_loc	25 2 118 16
	cmp	rcx, 256
	jne	.LBB6_19
.Ltmp502:
	.cv_loc	25 2 118 30
	.seh_startepilogue
	add	rsp, 1024
	pop	rdi
	pop	rsi
	pop	rbp
	.seh_endepilogue
	vzeroupper
	ret
.Ltmp503:
.Lfunc_end6:
	.seh_endproc

	.def	proof.RadicalPredictor.getCumFreqsRC;
	.scl	3;
	.type	32;
	.endef
	.p2align	4
proof.RadicalPredictor.getCumFreqsRC:
.Lfunc_begin7:
	.cv_func_id 26
	.cv_loc	26 2 94 0
.seh_proc proof.RadicalPredictor.getCumFreqsRC
	push	rbp
	.seh_pushreg rbp
.Ltmp504:
	push	rsi
	.seh_pushreg rsi
	push	rdi
	.seh_pushreg rdi
	sub	rsp, 1024
	.seh_stackalloc 1024
	lea	rbp, [rsp + 128]
	.seh_setframe rbp, 128
	.seh_endprologue
	.cv_loc	26 2 95 32
	vbroadcastss	ymm0, dword ptr [rcx + 24]
	vmovups	ymmword ptr [rbp - 128], ymm0
	vmovups	ymmword ptr [rbp - 96], ymm0
	vmovups	ymmword ptr [rbp - 64], ymm0
	vmovups	ymmword ptr [rbp - 32], ymm0
	vmovups	ymmword ptr [rbp], ymm0
	vmovups	ymmword ptr [rbp + 32], ymm0
	vmovups	ymmword ptr [rbp + 64], ymm0
	vmovups	ymmword ptr [rbp + 96], ymm0
	vmovups	ymmword ptr [rbp + 128], ymm0
	vmovups	ymmword ptr [rbp + 160], ymm0
	vmovups	ymmword ptr [rbp + 192], ymm0
	vmovups	ymmword ptr [rbp + 224], ymm0
	vmovups	ymmword ptr [rbp + 256], ymm0
	vmovups	ymmword ptr [rbp + 288], ymm0
	vmovups	ymmword ptr [rbp + 320], ymm0
	vmovups	ymmword ptr [rbp + 352], ymm0
	vmovups	ymmword ptr [rbp + 384], ymm0
	vmovups	ymmword ptr [rbp + 416], ymm0
	vmovups	ymmword ptr [rbp + 448], ymm0
	vmovups	ymmword ptr [rbp + 480], ymm0
	vmovups	ymmword ptr [rbp + 512], ymm0
	vmovups	ymmword ptr [rbp + 544], ymm0
	vmovups	ymmword ptr [rbp + 576], ymm0
	vmovups	ymmword ptr [rbp + 608], ymm0
	vmovups	ymmword ptr [rbp + 640], ymm0
	vmovups	ymmword ptr [rbp + 672], ymm0
	vmovups	ymmword ptr [rbp + 704], ymm0
	vmovups	ymmword ptr [rbp + 736], ymm0
	vmovups	ymmword ptr [rbp + 768], ymm0
	vmovups	ymmword ptr [rbp + 800], ymm0
	vmovups	ymmword ptr [rbp + 832], ymm0
	vmovups	ymmword ptr [rbp + 864], ymm0
	.cv_loc	26 2 96 35
	mov	r9, qword ptr [rcx]
.Ltmp505:
	.cv_loc	26 2 96 27
	test	r9, r9
	je	.LBB7_18
.Ltmp506:
	movzx	eax, dl
	mov	edx, r9d
.Ltmp507:
	and	edx, 3
	cmp	r9, 4
	jae	.LBB7_3
.Ltmp508:
	.cv_loc	26 2 97 22
	xor	r9d, r9d
.Ltmp509:
	test	rdx, rdx
	jne	.LBB7_14
	jmp	.LBB7_18
.Ltmp510:
.LBB7_3:
	.cv_loc	26 2 96 27
	and	r9, -4
	lea	r10, [rcx + 76]
	mov	r11, r9
	jmp	.LBB7_4
.Ltmp511:
	.p2align	4
.LBB7_11:
	.cv_loc	26 2 96 27
	add	r10, 48
	add	r11, -4
	je	.LBB7_12
.Ltmp512:
.LBB7_4:
	.cv_loc	26 2 97 22
	cmp	dword ptr [r10 - 44], eax
	je	.LBB7_21
.Ltmp513:
	.cv_loc	26 2 97 22
	cmp	dword ptr [r10 - 32], eax
	je	.LBB7_6
.Ltmp514:
.LBB7_7:
	.cv_loc	26 2 97 22
	cmp	dword ptr [r10 - 20], eax
	je	.LBB7_8
.Ltmp515:
.LBB7_9:
	.cv_loc	26 2 97 22
	cmp	dword ptr [r10 - 8], eax
	jne	.LBB7_11
	jmp	.LBB7_10
.Ltmp516:
	.p2align	4
.LBB7_21:
	.cv_loc	26 2 96 27
	movzx	esi, byte ptr [r10 - 36]
.Ltmp517:
	mov	edi, dword ptr [r10 - 40]
.Ltmp518:
	.cv_loc	26 2 98 34
	add	dword ptr [rbp + 4*rsi - 128], edi
.Ltmp519:
	.cv_loc	26 2 97 22
	cmp	dword ptr [r10 - 32], eax
	jne	.LBB7_7
.Ltmp520:
.LBB7_6:
	.cv_loc	26 2 96 27
	movzx	esi, byte ptr [r10 - 24]
.Ltmp521:
	mov	edi, dword ptr [r10 - 28]
.Ltmp522:
	.cv_loc	26 2 98 34
	add	dword ptr [rbp + 4*rsi - 128], edi
.Ltmp523:
	.cv_loc	26 2 97 22
	cmp	dword ptr [r10 - 20], eax
	jne	.LBB7_9
.Ltmp524:
.LBB7_8:
	.cv_loc	26 2 96 27
	movzx	esi, byte ptr [r10 - 12]
.Ltmp525:
	mov	edi, dword ptr [r10 - 16]
.Ltmp526:
	.cv_loc	26 2 98 34
	add	dword ptr [rbp + 4*rsi - 128], edi
.Ltmp527:
	.cv_loc	26 2 97 22
	cmp	dword ptr [r10 - 8], eax
	jne	.LBB7_11
.Ltmp528:
.LBB7_10:
	.cv_loc	26 2 96 27
	movzx	esi, byte ptr [r10]
.Ltmp529:
	mov	edi, dword ptr [r10 - 4]
.Ltmp530:
	.cv_loc	26 2 98 34
	add	dword ptr [rbp + 4*rsi - 128], edi
	jmp	.LBB7_11
.Ltmp531:
.LBB7_12:
	.cv_loc	26 2 97 22
	shl	r9, 2
	lea	r9, [r9 + 2*r9]
	.cv_loc	26 2 97 22
	test	rdx, rdx
	je	.LBB7_18
.Ltmp532:
.LBB7_14:
	.cv_loc	26 2 97 22
	add	rcx, r9
.Ltmp533:
	add	rcx, 40
	shl	edx, 2
	lea	rdx, [rdx + 2*rdx]
	xor	r9d, r9d
	jmp	.LBB7_15
.Ltmp534:
	.p2align	4
.LBB7_17:
	.cv_loc	26 2 96 27
	add	r9, 12
	cmp	rdx, r9
	je	.LBB7_18
.Ltmp535:
.LBB7_15:
	.cv_loc	26 2 97 22
	cmp	dword ptr [rcx + r9 - 8], eax
	jne	.LBB7_17
.Ltmp536:
	.cv_loc	26 2 96 27
	movzx	r10d, byte ptr [rcx + r9]
.Ltmp537:
	mov	r11d, dword ptr [rcx + r9 - 4]
.Ltmp538:
	.cv_loc	26 2 98 34
	add	dword ptr [rbp + 4*r10 - 128], r11d
	jmp	.LBB7_17
.Ltmp539:
.LBB7_18:
	.cv_loc	26 2 101 18
	mov	dword ptr [r8], 0
	xor	eax, eax
.Ltmp540:
	xor	ecx, ecx
.Ltmp541:
	.p2align	4
.LBB7_19:
	.cv_loc	26 2 104 45
	add	eax, dword ptr [rbp + 4*rcx - 128]
	mov	dword ptr [r8 + 4*rcx + 4], eax
.Ltmp542:
	add	eax, dword ptr [rbp + 4*rcx - 124]
	mov	dword ptr [r8 + 4*rcx + 8], eax
.Ltmp543:
	add	eax, dword ptr [rbp + 4*rcx - 120]
	mov	dword ptr [r8 + 4*rcx + 12], eax
.Ltmp544:
	add	eax, dword ptr [rbp + 4*rcx - 116]
	mov	dword ptr [r8 + 4*rcx + 16], eax
.Ltmp545:
	add	eax, dword ptr [rbp + 4*rcx - 112]
	mov	dword ptr [r8 + 4*rcx + 20], eax
.Ltmp546:
	add	eax, dword ptr [rbp + 4*rcx - 108]
	mov	dword ptr [r8 + 4*rcx + 24], eax
.Ltmp547:
	add	eax, dword ptr [rbp + 4*rcx - 104]
	mov	dword ptr [r8 + 4*rcx + 28], eax
.Ltmp548:
	add	eax, dword ptr [rbp + 4*rcx - 100]
	mov	dword ptr [r8 + 4*rcx + 32], eax
.Ltmp549:
	.cv_loc	26 2 104 25
	add	rcx, 8
.Ltmp550:
	.cv_loc	26 2 103 16
	cmp	rcx, 256
	jne	.LBB7_19
.Ltmp551:
	.cv_loc	26 2 103 30
	.seh_startepilogue
	add	rsp, 1024
	pop	rdi
	pop	rsi
	pop	rbp
	.seh_endepilogue
	vzeroupper
	ret
.Ltmp552:
.Lfunc_end7:
	.seh_endproc

	.def	proof.wasm_decode;
	.scl	3;
	.type	32;
	.endef
	.p2align	4
proof.wasm_decode:
.Lfunc_begin8:
	.cv_func_id 27
	.cv_loc	27 2 371 0
.seh_proc proof.wasm_decode
	push	rbp
	.seh_pushreg rbp
.Ltmp553:
	push	rsi
	.seh_pushreg rsi
	sub	rsp, 40
	.seh_stackalloc 40
	lea	rbp, [rsp + 32]
	.seh_setframe rbp, 32
	.seh_endprologue
	.cv_loc	27 2 372 11
	lea	rsi, [rip + proof.global_decoded_buf]
	mov	r9, rsi
	call	proof.decode
.Ltmp554:
	.cv_loc	27 2 373 5
	mov	rax, rsi
	.seh_startepilogue
	add	rsp, 40
	pop	rsi
	pop	rbp
	.seh_endepilogue
	ret
.Ltmp555:
.Lfunc_end8:
	.seh_endproc

	.def	proof.wasm_get_encoded_bits;
	.scl	3;
	.type	32;
	.endef
	.p2align	4
proof.wasm_get_encoded_bits:
.Lfunc_begin9:
	.cv_func_id 28
	.cv_loc	28 2 367 0
.seh_proc proof.wasm_get_encoded_bits
	push	rbp
	.seh_pushreg rbp
	mov	rbp, rsp
	.seh_setframe rbp, 0
	.seh_endprologue
.Ltmp556:
	.cv_loc	28 2 368 25
	mov	rax, qword ptr [rip + proof.global_writer]
	.cv_loc	28 2 368 5
	.seh_startepilogue
	pop	rbp
	.seh_endepilogue
	ret
.Ltmp557:
.Lfunc_end9:
	.seh_endproc

	.def	proof.wasm_encode;
	.scl	3;
	.type	32;
	.endef
	.p2align	4
proof.wasm_encode:
.Lfunc_begin10:
	.cv_func_id 29
	.cv_loc	29 2 360 0
.seh_proc proof.wasm_encode
	push	rbp
	.seh_pushreg rbp
	push	rsi
	.seh_pushreg rsi
	push	rdi
	.seh_pushreg rdi
	push	rbx
	.seh_pushreg rbx
	sub	rsp, 40
	.seh_stackalloc 40
	lea	rbp, [rsp + 32]
	.seh_setframe rbp, 32
	.seh_endprologue
	mov	rsi, rdx
	mov	rdi, rcx
.Ltmp558:
	.cv_loc	29 2 362 27
	lea	rbx, [rip + proof.global_writer]
	mov	r8d, 10248
	mov	rcx, rbx
.Ltmp559:
	xor	edx, edx
.Ltmp560:
	call	memset
	.cv_loc	29 2 363 11
	mov	rcx, rdi
	mov	rdx, rsi
	mov	r8, rbx
	call	proof.encode
	.cv_loc	29 2 364 5
	lea	rax, [rip + proof.global_writer+8]
	.seh_startepilogue
	add	rsp, 40
	pop	rbx
	pop	rdi
.Ltmp561:
	pop	rsi
.Ltmp562:
	pop	rbp
	.seh_endepilogue
	ret
.Ltmp563:
.Lfunc_end10:
	.seh_endproc

	.section	.rdata,"dr"
__anon_5057:
	.byte	1
	.byte	2
	.byte	3
	.byte	4
	.byte	5
	.byte	6
	.byte	8
	.byte	0
	.byte	15
	.byte	1
	.byte	0
	.byte	15
	.zero	6
	.byte	15
	.byte	15
	.byte	15
	.byte	15
	.byte	15
	.byte	15
	.byte	4
	.byte	5
	.byte	6
	.byte	7
	.byte	8
	.byte	9

	.section	.rdata$T,"dr"
	.p2align	3, 0x0
os.windows.tls._tls_used:
	.quad	os.windows.tls._tls_start
	.quad	os.windows.tls._tls_end
	.quad	os.windows.tls._tls_index
	.quad	os.windows.tls.__xl_a+8
	.long	0
	.long	0

	.section	.CRT$XLZ,"dw"
	.p2align	3, 0x0
os.windows.tls.__xl_z:
	.quad	0

	.section	.tls$ZZZ,"dw"
	.p2align	3, 0x0
os.windows.tls._tls_end:
	.quad	0

	.section	.tls,"dw"
	.p2align	3, 0x0
os.windows.tls._tls_start:
	.quad	0

	.section	.CRT$XLA,"dw"
	.p2align	3, 0x0
os.windows.tls.__xl_a:
	.quad	0

	.data
	.p2align	2, 0x0
os.windows.tls._tls_index:
	.long	4294967295

	.lcomm	proof.global_writer,10248,8
	.lcomm	proof.global_decoded_buf,6000
	.globl	wWinMainCRTStartup
	.def	wWinMainCRTStartup;
	.scl	2;
	.type	32;
	.endef
wWinMainCRTStartup = start.WinStartup
	.globl	run_verification
	.def	run_verification;
	.scl	2;
	.type	32;
	.endef
run_verification = proof.run_verification
	.globl	_tls_used
_tls_used = os.windows.tls._tls_used
	.globl	__xl_z
__xl_z = os.windows.tls.__xl_z
	.globl	_tls_end
_tls_end = os.windows.tls._tls_end
	.globl	_tls_start
_tls_start = os.windows.tls._tls_start
	.globl	__xl_a
__xl_a = os.windows.tls.__xl_a
	.globl	_tls_index
_tls_index = os.windows.tls._tls_index
	.globl	wasm_decode
	.def	wasm_decode;
	.scl	2;
	.type	32;
	.endef
wasm_decode = proof.wasm_decode
	.globl	wasm_get_encoded_bits
	.def	wasm_get_encoded_bits;
	.scl	2;
	.type	32;
	.endef
wasm_get_encoded_bits = proof.wasm_get_encoded_bits
	.globl	wasm_encode
	.def	wasm_encode;
	.scl	2;
	.type	32;
	.endef
wasm_encode = proof.wasm_encode
	.section	.debug$S,"dr"
	.p2align	2, 0x0
	.long	4
	.long	241
	.long	.Ltmp565-.Ltmp564
.Ltmp564:
	.short	.Ltmp567-.Ltmp566
.Ltmp566:
	.short	4353
	.long	0
	.byte	0
	.p2align	2, 0x0
.Ltmp567:
	.short	.Ltmp569-.Ltmp568
.Ltmp568:
	.short	4412
	.long	0
	.short	208
	.short	0
	.short	16
	.short	0
	.short	0
	.short	21010
	.short	0
	.short	0
	.short	0
	.asciz	"zig 0.16.0"
	.p2align	2, 0x0
.Ltmp569:
.Ltmp565:
	.p2align	2, 0x0
	.long	246
	.long	.Ltmp571-.Ltmp570
.Ltmp570:
	.long	0


	.long	4105
	.cv_filechecksumoffset	2
	.long	143


	.long	4118
	.cv_filechecksumoffset	2
	.long	25


	.long	4129
	.cv_filechecksumoffset	2
	.long	177


	.long	4132
	.cv_filechecksumoffset	2
	.long	185


	.long	4135
	.cv_filechecksumoffset	2
	.long	163


	.long	4138
	.cv_filechecksumoffset	2
	.long	150
.Ltmp571:
	.p2align	2, 0x0
	.long	241
	.long	.Ltmp573-.Ltmp572
.Ltmp572:
	.short	.Ltmp575-.Ltmp574
.Ltmp574:
	.short	4422
	.long	0
	.long	0
	.long	0
	.long	.Lfunc_end0-start.WinStartup
	.long	0
	.long	0
	.long	4141
	.secrel32	start.WinStartup
	.secidx	start.WinStartup
	.byte	137
	.asciz	"WinStartup"
	.p2align	2, 0x0
.Ltmp575:
	.short	.Ltmp577-.Ltmp576
.Ltmp576:
	.short	4114
	.long	40
	.long	0
	.long	0
	.long	0
	.long	0
	.short	0
	.long	1204232
	.p2align	2, 0x0
.Ltmp577:
	.short	2
	.short	4431
.Ltmp573:
	.p2align	2, 0x0
	.cv_linetable	0, start.WinStartup, .Lfunc_end0
	.long	241
	.long	.Ltmp579-.Ltmp578
.Ltmp578:
	.short	.Ltmp581-.Ltmp580
.Ltmp580:
	.short	4422
	.long	0
	.long	0
	.long	0
	.long	.Lfunc_end1-proof.run_verification
	.long	0
	.long	0
	.long	4143
	.secrel32	proof.run_verification
	.secidx	proof.run_verification
	.byte	129
	.asciz	"run_verification"
	.p2align	2, 0x0
.Ltmp581:
	.short	.Ltmp583-.Ltmp582
.Ltmp582:
	.short	4114
	.long	10328
	.long	0
	.long	0
	.long	16
	.long	0
	.short	0
	.long	1220608
	.p2align	2, 0x0
.Ltmp583:
	.short	.Ltmp585-.Ltmp584
.Ltmp584:
	.short	4456
	.long	1
	.long	4105
	.p2align	2, 0x0
.Ltmp585:
	.short	.Ltmp587-.Ltmp586
.Ltmp586:
	.short	4414
	.long	4102
	.short	0
	.asciz	"writer"
	.p2align	2, 0x0
.Ltmp587:
	.cv_def_range	 .Ltmp2 .Ltmp26, frame_ptr_rel, -88
	.short	.Ltmp589-.Ltmp588
.Ltmp588:
	.short	4414
	.long	4145
	.short	0
	.asciz	"decoded_buf"
	.p2align	2, 0x0
.Ltmp589:
	.cv_def_range	 .Ltmp2 .Ltmp26, frame_ptr_rel, 10162
	.short	.Ltmp591-.Ltmp590
.Ltmp590:
	.short	4414
	.long	35
	.short	0
	.asciz	"written_bytes"
	.p2align	2, 0x0
.Ltmp591:
	.cv_def_range	 .Ltmp4 .Ltmp5, reg, 331
	.short	.Ltmp593-.Ltmp592
.Ltmp592:
	.short	4359
	.long	35
	.byte	0x04, 0x00
	.asciz	"i"
	.p2align	2, 0x0
.Ltmp593:
	.short	.Ltmp595-.Ltmp594
.Ltmp594:
	.short	4355
	.long	0
	.long	0
	.long	.Ltmp25-.Ltmp5
	.secrel32	.Ltmp5
	.secidx	.Lfunc_begin1
	.byte	0
	.p2align	2, 0x0
.Ltmp595:
	.short	.Ltmp597-.Ltmp596
.Ltmp596:
	.short	4359
	.long	4144
	.byte	0x06, 0x00
	.asciz	"orig"
	.p2align	2, 0x0
.Ltmp597:
	.short	.Ltmp599-.Ltmp598
.Ltmp598:
	.short	4414
	.long	4147
	.short	256
	.asciz	"dec"
	.p2align	2, 0x0
.Ltmp599:
	.short	2
	.short	6
	.short	.Ltmp601-.Ltmp600
.Ltmp600:
	.short	4429
	.long	0
	.long	0
	.long	4105
	.cv_inline_linetable	2 2 143 .Lfunc_begin1 .Lfunc_end1
	.p2align	2, 0x0
.Ltmp601:
	.short	2
	.short	4430
	.short	2
	.short	4431
.Ltmp579:
	.p2align	2, 0x0
	.cv_linetable	1, proof.run_verification, .Lfunc_end1
	.long	241
	.long	.Ltmp603-.Ltmp602
.Ltmp602:
	.short	.Ltmp605-.Ltmp604
.Ltmp604:
	.short	4422
	.long	0
	.long	0
	.long	0
	.long	.Lfunc_end2-proof.decode
	.long	0
	.long	0
	.long	4155
	.secrel32	proof.decode
	.secidx	proof.decode
	.byte	129
	.asciz	"decode"
	.p2align	2, 0x0
.Ltmp605:
	.short	.Ltmp607-.Ltmp606
.Ltmp606:
	.short	4114
	.long	10384
	.long	0
	.long	0
	.long	56
	.long	0
	.short	0
	.long	1220608
	.p2align	2, 0x0
.Ltmp607:
	.short	.Ltmp609-.Ltmp608
.Ltmp608:
	.short	4456
	.long	3
	.long	4118
	.long	4129
	.long	4132
	.p2align	2, 0x0
.Ltmp609:
	.short	.Ltmp611-.Ltmp610
.Ltmp610:
	.short	4414
	.long	4128
	.short	257
	.asciz	"encoded_bytes"
	.p2align	2, 0x0
.Ltmp611:
	.short	.Ltmp613-.Ltmp612
.Ltmp612:
	.short	4414
	.long	35
	.short	1
	.asciz	"num_concepts"
	.p2align	2, 0x0
.Ltmp613:
	.cv_def_range	 .Lfunc_begin2 .Ltmp27, reg, 336
	.cv_def_range	 .Ltmp27 .Lfunc_end2, frame_ptr_rel, 10232
	.short	.Ltmp615-.Ltmp614
.Ltmp614:
	.short	4414
	.long	4154
	.short	257
	.asciz	"decoded"
	.p2align	2, 0x0
.Ltmp615:
	.short	.Ltmp617-.Ltmp616
.Ltmp616:
	.short	4414
	.long	4113
	.short	0
	.asciz	"pred"
	.p2align	2, 0x0
.Ltmp617:
	.cv_def_range	 .Ltmp28 .Ltmp143, frame_ptr_rel, -80
	.short	.Ltmp619-.Ltmp618
.Ltmp618:
	.short	4359
	.long	4119
	.byte	0x00, 0x00
	.asciz	"r"
	.p2align	2, 0x0
.Ltmp619:
	.short	.Ltmp621-.Ltmp620
.Ltmp620:
	.short	4359
	.long	117
	.byte	0x00, 0x00
	.asciz	"value"
	.p2align	2, 0x0
.Ltmp621:
	.short	.Ltmp623-.Ltmp622
.Ltmp622:
	.short	4359
	.long	35
	.byte	0x00, 0x00
	.asciz	"i"
	.p2align	2, 0x0
.Ltmp623:
	.short	.Ltmp625-.Ltmp624
.Ltmp624:
	.short	4359
	.long	117
	.byte	0x00, 0x00
	.asciz	"low"
	.p2align	2, 0x0
.Ltmp625:
	.short	.Ltmp627-.Ltmp626
.Ltmp626:
	.short	4359
	.long	117
	.byte	0x00, 0x80, 0xff
	.asciz	"high"
	.p2align	2, 0x0
.Ltmp627:
	.short	.Ltmp629-.Ltmp628
.Ltmp628:
	.short	4359
	.long	35
	.byte	0x00, 0x00
	.asciz	"c_idx"
	.p2align	2, 0x0
.Ltmp629:
	.short	.Ltmp631-.Ltmp630
.Ltmp630:
	.short	4414
	.long	4156
	.short	0
	.asciz	"symbols"
	.p2align	2, 0x0
.Ltmp631:
	.cv_def_range	 .Ltmp57 .Ltmp58 .Ltmp61 .Ltmp115 .Ltmp118 .Ltmp134 .Ltmp137 .Ltmp139, frame_ptr_rel, 10244
	.short	.Ltmp633-.Ltmp632
.Ltmp632:
	.short	4414
	.long	32
	.short	0
	.asciz	"prev_rc"
	.p2align	2, 0x0
.Ltmp633:
	.cv_def_range	 .Ltmp56 .Ltmp60 .Ltmp63 .Ltmp139, frame_ptr_rel, 10241
	.cv_def_range	 .Ltmp62 .Ltmp63, reg, 1
	.short	.Ltmp635-.Ltmp634
.Ltmp634:
	.short	4414
	.long	32
	.short	0
	.asciz	"prev_rf"
	.p2align	2, 0x0
.Ltmp635:
	.cv_def_range	 .Ltmp56 .Ltmp60 .Ltmp65 .Ltmp139, frame_ptr_rel, 10242
	.cv_def_range	 .Ltmp64 .Ltmp65, reg, 1
	.short	.Ltmp637-.Ltmp636
.Ltmp636:
	.short	4414
	.long	32
	.short	0
	.asciz	"prev_ra"
	.p2align	2, 0x0
.Ltmp637:
	.cv_def_range	 .Ltmp56 .Ltmp60 .Ltmp67 .Ltmp139, frame_ptr_rel, 10243
	.cv_def_range	 .Ltmp66 .Ltmp67, reg, 1
	.short	.Ltmp639-.Ltmp638
.Ltmp638:
	.short	4359
	.long	35
	.byte	0x00, 0x00
	.asciz	"step"
	.p2align	2, 0x0
.Ltmp639:
	.short	.Ltmp641-.Ltmp640
.Ltmp640:
	.short	4414
	.long	35
	.short	0
	.asciz	"total"
	.p2align	2, 0x0
.Ltmp641:
	.cv_def_range	 .Ltmp77 .Ltmp139, reg, 360
	.short	.Ltmp643-.Ltmp642
.Ltmp642:
	.short	4414
	.long	35
	.short	0
	.asciz	"range_width"
	.p2align	2, 0x0
.Ltmp643:
	.cv_def_range	 .Ltmp78 .Ltmp105, reg, 330
	.short	.Ltmp645-.Ltmp644
.Ltmp644:
	.short	4414
	.long	35
	.short	0
	.asciz	"scaled_val"
	.p2align	2, 0x0
.Ltmp645:
	.cv_def_range	 .Ltmp82 .Ltmp95, reg, 328
	.short	.Ltmp647-.Ltmp646
.Ltmp646:
	.short	4359
	.long	32
	.byte	0x00, 0x00
	.asciz	"sym"
	.p2align	2, 0x0
.Ltmp647:
	.short	.Ltmp649-.Ltmp648
.Ltmp648:
	.short	4359
	.long	116
	.byte	0x00, 0x00
	.asciz	"l"
	.p2align	2, 0x0
.Ltmp649:
	.short	.Ltmp651-.Ltmp650
.Ltmp650:
	.short	4359
	.long	116
	.byte	0xff, 0x00
	.asciz	"rr"
	.p2align	2, 0x0
.Ltmp651:
	.short	.Ltmp653-.Ltmp652
.Ltmp652:
	.short	4414
	.long	35
	.short	256
	.asciz	"sym_idx"
	.p2align	2, 0x0
.Ltmp653:
	.short	.Ltmp655-.Ltmp654
.Ltmp654:
	.short	4414
	.long	117
	.short	256
	.asciz	"cum_low"
	.p2align	2, 0x0
.Ltmp655:
	.short	.Ltmp657-.Ltmp656
.Ltmp656:
	.short	4414
	.long	117
	.short	256
	.asciz	"cum_high"
	.p2align	2, 0x0
.Ltmp657:
	.short	.Ltmp659-.Ltmp658
.Ltmp658:
	.short	4355
	.long	0
	.long	0
	.long	.Ltmp91-.Ltmp85
	.secrel32	.Ltmp85
	.secidx	.Lfunc_begin2
	.byte	0
	.p2align	2, 0x0
.Ltmp659:
	.short	.Ltmp661-.Ltmp660
.Ltmp660:
	.short	4414
	.long	116
	.short	0
	.asciz	"mid"
	.p2align	2, 0x0
.Ltmp661:
	.cv_def_range	 .Ltmp86 .Ltmp92, reg, 362
	.short	2
	.short	6
	.short	.Ltmp663-.Ltmp662
.Ltmp662:
	.short	4429
	.long	0
	.long	0
	.long	4118
	.cv_inline_linetable	4 2 25 .Lfunc_begin2 .Lfunc_end2
	.p2align	2, 0x0
.Ltmp663:
	.short	.Ltmp665-.Ltmp664
.Ltmp664:
	.short	4414
	.long	117
	.short	257
	.asciz	"alpha"
	.p2align	2, 0x0
.Ltmp665:
	.short	.Ltmp667-.Ltmp666
.Ltmp666:
	.short	4414
	.long	117
	.short	257
	.asciz	"weight"
	.p2align	2, 0x0
.Ltmp667:
	.short	2
	.short	4430
	.short	.Ltmp669-.Ltmp668
.Ltmp668:
	.short	4429
	.long	0
	.long	0
	.long	4129
	.cv_inline_linetable	5 2 177 .Lfunc_begin2 .Lfunc_end2
	.p2align	2, 0x0
.Ltmp669:
	.short	.Ltmp671-.Ltmp670
.Ltmp670:
	.short	4414
	.long	4128
	.short	257
	.asciz	"buffer"
	.p2align	2, 0x0
.Ltmp671:
	.short	2
	.short	4430
	.short	.Ltmp673-.Ltmp672
.Ltmp672:
	.short	4429
	.long	0
	.long	0
	.long	4132
	.cv_inline_linetable	6 2 185 .Lfunc_begin2 .Lfunc_end2
	.p2align	2, 0x0
.Ltmp673:
	.short	.Ltmp675-.Ltmp674
.Ltmp674:
	.short	4414
	.long	4120
	.short	257
	.asciz	"self"
	.p2align	2, 0x0
.Ltmp675:
	.short	.Ltmp677-.Ltmp676
.Ltmp676:
	.short	4414
	.long	35
	.short	0
	.asciz	"byte_pos"
	.p2align	2, 0x0
.Ltmp677:
	.cv_def_range	 .Ltmp39 .Ltmp40, reg, 330
	.cv_def_range	 .Ltmp50 .Ltmp51, reg, 331
	.short	.Ltmp679-.Ltmp678
.Ltmp678:
	.short	4414
	.long	32
	.short	256
	.asciz	"bit_pos"
	.p2align	2, 0x0
.Ltmp679:
	.short	.Ltmp681-.Ltmp680
.Ltmp680:
	.short	4414
	.long	32
	.short	0
	.asciz	"bit"
	.p2align	2, 0x0
.Ltmp681:
	.cv_def_range	 .Ltmp52 .Ltmp54, reg, 19
	.short	2
	.short	4430
	.short	.Ltmp683-.Ltmp682
.Ltmp682:
	.short	4429
	.long	0
	.long	0
	.long	4132
	.cv_inline_linetable	7 2 185 .Lfunc_begin2 .Lfunc_end2
	.p2align	2, 0x0
.Ltmp683:
	.short	2
	.short	4430
	.short	.Ltmp685-.Ltmp684
.Ltmp684:
	.short	4429
	.long	0
	.long	0
	.long	4132
	.cv_inline_linetable	8 2 185 .Lfunc_begin2 .Lfunc_end2
	.p2align	2, 0x0
.Ltmp685:
	.short	.Ltmp687-.Ltmp686
.Ltmp686:
	.short	4414
	.long	4120
	.short	257
	.asciz	"self"
	.p2align	2, 0x0
.Ltmp687:
	.short	.Ltmp689-.Ltmp688
.Ltmp688:
	.short	4414
	.long	35
	.short	0
	.asciz	"byte_pos"
	.p2align	2, 0x0
.Ltmp689:
	.cv_def_range	 .Ltmp131 .Ltmp132, reg, 328
	.short	.Ltmp691-.Ltmp690
.Ltmp690:
	.short	4414
	.long	32
	.short	256
	.asciz	"bit_pos"
	.p2align	2, 0x0
.Ltmp691:
	.short	.Ltmp693-.Ltmp692
.Ltmp692:
	.short	4414
	.long	32
	.short	0
	.asciz	"bit"
	.p2align	2, 0x0
.Ltmp693:
	.cv_def_range	 .Ltmp133 .Ltmp134, reg, 17
	.short	2
	.short	4430
	.short	2
	.short	4431
.Ltmp603:
	.p2align	2, 0x0
	.cv_linetable	3, proof.decode, .Lfunc_end2
	.long	241
	.long	.Ltmp695-.Ltmp694
.Ltmp694:
	.short	.Ltmp697-.Ltmp696
.Ltmp696:
	.short	4422
	.long	0
	.long	0
	.long	0
	.long	.Lfunc_end3-proof.encode
	.long	0
	.long	0
	.long	4161
	.secrel32	proof.encode
	.secidx	proof.encode
	.byte	129
	.asciz	"encode"
	.p2align	2, 0x0
.Ltmp697:
	.short	.Ltmp699-.Ltmp698
.Ltmp698:
	.short	4114
	.long	10384
	.long	0
	.long	0
	.long	56
	.long	0
	.short	0
	.long	1220608
	.p2align	2, 0x0
.Ltmp699:
	.short	.Ltmp701-.Ltmp700
.Ltmp700:
	.short	4456
	.long	2
	.long	4118
	.long	4135
	.p2align	2, 0x0
.Ltmp701:
	.short	.Ltmp703-.Ltmp702
.Ltmp702:
	.short	4414
	.long	4160
	.short	257
	.asciz	"concepts"
	.p2align	2, 0x0
.Ltmp703:
	.short	.Ltmp705-.Ltmp704
.Ltmp704:
	.short	4414
	.long	4097
	.short	1
	.asciz	"writer"
	.p2align	2, 0x0
.Ltmp705:
	.cv_def_range	 .Lfunc_begin3 .Ltmp148, reg, 336
	.cv_def_range	 .Ltmp148 .Ltmp317 .Ltmp318 .Lfunc_end3, reg, 332
	.short	.Ltmp707-.Ltmp706
.Ltmp706:
	.short	4414
	.long	4113
	.short	0
	.asciz	"pred"
	.p2align	2, 0x0
.Ltmp707:
	.cv_def_range	 .Ltmp144 .Ltmp347, frame_ptr_rel, -80
	.short	.Ltmp709-.Ltmp708
.Ltmp708:
	.short	4414
	.long	4162
	.short	0
	.asciz	"cum_freqs"
	.p2align	2, 0x0
.Ltmp709:
	.cv_def_range	 .Ltmp144 .Ltmp347, frame_ptr_rel, 9188
	.short	.Ltmp711-.Ltmp710
.Ltmp710:
	.short	4359
	.long	117
	.byte	0x00, 0x00
	.asciz	"low"
	.p2align	2, 0x0
.Ltmp711:
	.short	.Ltmp713-.Ltmp712
.Ltmp712:
	.short	4359
	.long	117
	.byte	0x00, 0x80, 0xff
	.asciz	"high"
	.p2align	2, 0x0
.Ltmp713:
	.short	.Ltmp715-.Ltmp714
.Ltmp714:
	.short	4359
	.long	117
	.byte	0x00, 0x00
	.asciz	"underflow_bits"
	.p2align	2, 0x0
.Ltmp715:
	.short	.Ltmp717-.Ltmp716
.Ltmp716:
	.short	4414
	.long	32
	.short	0
	.asciz	"rc"
	.p2align	2, 0x0
.Ltmp717:
	.cv_def_range	 .Ltmp149 .Ltmp152 .Ltmp158 .Ltmp277, reg, 348
	.short	.Ltmp719-.Ltmp718
.Ltmp718:
	.short	4414
	.long	32
	.short	0
	.asciz	"rf"
	.p2align	2, 0x0
.Ltmp719:
	.cv_def_range	 .Ltmp149 .Ltmp152 .Ltmp160 .Ltmp277, reg, 349
	.short	.Ltmp721-.Ltmp720
.Ltmp720:
	.short	4414
	.long	32
	.short	0
	.asciz	"ra"
	.p2align	2, 0x0
.Ltmp721:
	.cv_def_range	 .Ltmp149 .Ltmp152 .Ltmp163 .Ltmp277, frame_ptr_rel, 10247
	.cv_def_range	 .Ltmp162 .Ltmp163, reg, 3
	.short	.Ltmp723-.Ltmp722
.Ltmp722:
	.short	4414
	.long	4156
	.short	0
	.asciz	"symbols"
	.p2align	2, 0x0
.Ltmp723:
	.cv_def_range	 .Ltmp149 .Ltmp152 .Ltmp163 .Ltmp277, reg_rel, 334, 33, 10247
	.cv_def_range	 .Ltmp149 .Ltmp152 .Ltmp162 .Ltmp277, subfield_reg, 348, 0
	.cv_def_range	 .Ltmp149 .Ltmp152 .Ltmp162 .Ltmp277, subfield_reg, 349, 1
	.cv_def_range	 .Ltmp162 .Ltmp163, subfield_reg, 3, 2
	.short	.Ltmp725-.Ltmp724
.Ltmp724:
	.short	4414
	.long	32
	.short	0
	.asciz	"prev_rc"
	.p2align	2, 0x0
.Ltmp725:
	.cv_def_range	 .Ltmp149 .Ltmp152 .Ltmp165 .Ltmp277, frame_ptr_rel, 10244
	.cv_def_range	 .Ltmp164 .Ltmp165, reg, 1
	.short	.Ltmp727-.Ltmp726
.Ltmp726:
	.short	4414
	.long	32
	.short	0
	.asciz	"prev_rf"
	.p2align	2, 0x0
.Ltmp727:
	.cv_def_range	 .Ltmp149 .Ltmp152 .Ltmp167 .Ltmp277, frame_ptr_rel, 10245
	.cv_def_range	 .Ltmp166 .Ltmp167, reg, 1
	.short	.Ltmp729-.Ltmp728
.Ltmp728:
	.short	4414
	.long	32
	.short	0
	.asciz	"prev_ra"
	.p2align	2, 0x0
.Ltmp729:
	.cv_def_range	 .Ltmp149 .Ltmp152 .Ltmp169 .Ltmp277, frame_ptr_rel, 10246
	.cv_def_range	 .Ltmp168 .Ltmp169, reg, 1
	.short	.Ltmp731-.Ltmp730
.Ltmp730:
	.short	4359
	.long	35
	.byte	0x00, 0x00
	.asciz	"step"
	.p2align	2, 0x0
.Ltmp731:
	.short	.Ltmp733-.Ltmp732
.Ltmp732:
	.short	4414
	.long	4147
	.short	0
	.asciz	"c"
	.p2align	2, 0x0
.Ltmp733:
	.cv_def_range	 .Ltmp154 .Ltmp157, subfield_reg, 348, 0
	.cv_def_range	 .Ltmp155 .Ltmp159, subfield_reg, 349, 2
	.cv_def_range	 .Ltmp156 .Ltmp161, subfield_reg, 3, 4
	.short	.Ltmp735-.Ltmp734
.Ltmp734:
	.short	4355
	.long	0
	.long	0
	.long	.Ltmp277-.Ltmp172
	.secrel32	.Ltmp172
	.secidx	.Lfunc_begin3
	.byte	0
	.p2align	2, 0x0
.Ltmp735:
	.short	.Ltmp737-.Ltmp736
.Ltmp736:
	.short	4414
	.long	35
	.short	256
	.asciz	"sym"
	.p2align	2, 0x0
.Ltmp737:
	.short	.Ltmp739-.Ltmp738
.Ltmp738:
	.short	4414
	.long	117
	.short	256
	.asciz	"total"
	.p2align	2, 0x0
.Ltmp739:
	.short	.Ltmp741-.Ltmp740
.Ltmp740:
	.short	4414
	.long	117
	.short	256
	.asciz	"cum_low"
	.p2align	2, 0x0
.Ltmp741:
	.short	.Ltmp743-.Ltmp742
.Ltmp742:
	.short	4414
	.long	117
	.short	256
	.asciz	"cum_high"
	.p2align	2, 0x0
.Ltmp743:
	.short	.Ltmp745-.Ltmp744
.Ltmp744:
	.short	4414
	.long	35
	.short	0
	.asciz	"range_width"
	.p2align	2, 0x0
.Ltmp745:
	.cv_def_range	 .Ltmp182 .Ltmp189, reg, 330
	.short	2
	.short	6
	.short	.Ltmp747-.Ltmp746
.Ltmp746:
	.short	4429
	.long	0
	.long	0
	.long	4118
	.cv_inline_linetable	10 2 25 .Lfunc_begin3 .Lfunc_end3
	.p2align	2, 0x0
.Ltmp747:
	.short	.Ltmp749-.Ltmp748
.Ltmp748:
	.short	4414
	.long	117
	.short	257
	.asciz	"alpha"
	.p2align	2, 0x0
.Ltmp749:
	.short	.Ltmp751-.Ltmp750
.Ltmp750:
	.short	4414
	.long	117
	.short	257
	.asciz	"weight"
	.p2align	2, 0x0
.Ltmp751:
	.short	2
	.short	4430
	.short	.Ltmp753-.Ltmp752
.Ltmp752:
	.short	4429
	.long	0
	.long	0
	.long	4135
	.cv_inline_linetable	11 2 163 .Lfunc_begin3 .Lfunc_end3
	.p2align	2, 0x0
.Ltmp753:
	.short	.Ltmp755-.Ltmp754
.Ltmp754:
	.short	4414
	.long	4097
	.short	1
	.asciz	"self"
	.p2align	2, 0x0
.Ltmp755:
	.cv_def_range	 .Ltmp202 .Ltmp235, reg, 332
	.short	.Ltmp757-.Ltmp756
.Ltmp756:
	.short	4414
	.long	1653
	.short	257
	.asciz	"underflow_bits"
	.p2align	2, 0x0
.Ltmp757:
	.short	.Ltmp759-.Ltmp758
.Ltmp758:
	.short	4414
	.long	32
	.short	257
	.asciz	"bit"
	.p2align	2, 0x0
.Ltmp759:
	.short	.Ltmp761-.Ltmp760
.Ltmp760:
	.short	4429
	.long	0
	.long	0
	.long	4138
	.cv_inline_linetable	12 2 150 .Lfunc_begin3 .Lfunc_end3
	.p2align	2, 0x0
.Ltmp761:
	.short	.Ltmp763-.Ltmp762
.Ltmp762:
	.short	4414
	.long	4097
	.short	1
	.asciz	"self"
	.p2align	2, 0x0
.Ltmp763:
	.cv_def_range	 .Ltmp202 .Ltmp210, reg, 332
	.short	.Ltmp765-.Ltmp764
.Ltmp764:
	.short	4414
	.long	32
	.short	257
	.asciz	"bit"
	.p2align	2, 0x0
.Ltmp765:
	.short	.Ltmp767-.Ltmp766
.Ltmp766:
	.short	4414
	.long	35
	.short	0
	.asciz	"byte_pos"
	.p2align	2, 0x0
.Ltmp767:
	.cv_def_range	 .Ltmp207 .Ltmp210, reg, 331
	.short	.Ltmp769-.Ltmp768
.Ltmp768:
	.short	4414
	.long	32
	.short	256
	.asciz	"bit_pos"
	.p2align	2, 0x0
.Ltmp769:
	.short	2
	.short	4430
	.short	.Ltmp771-.Ltmp770
.Ltmp770:
	.short	4429
	.long	0
	.long	0
	.long	4138
	.cv_inline_linetable	13 2 150 .Lfunc_begin3 .Lfunc_end3
	.p2align	2, 0x0
.Ltmp771:
	.short	.Ltmp773-.Ltmp772
.Ltmp772:
	.short	4414
	.long	4097
	.short	1
	.asciz	"self"
	.p2align	2, 0x0
.Ltmp773:
	.cv_def_range	 .Ltmp213 .Ltmp222 .Ltmp224 .Ltmp235, reg, 332
	.short	.Ltmp775-.Ltmp774
.Ltmp774:
	.short	4414
	.long	32
	.short	257
	.asciz	"bit"
	.p2align	2, 0x0
.Ltmp775:
	.short	.Ltmp777-.Ltmp776
.Ltmp776:
	.short	4414
	.long	35
	.short	0
	.asciz	"byte_pos"
	.p2align	2, 0x0
.Ltmp777:
	.cv_def_range	 .Ltmp217 .Ltmp220, reg, 331
	.cv_def_range	 .Ltmp226 .Ltmp228 .Ltmp233 .Ltmp235, reg, 336
	.short	.Ltmp779-.Ltmp778
.Ltmp778:
	.short	4414
	.long	32
	.short	256
	.asciz	"bit_pos"
	.p2align	2, 0x0
.Ltmp779:
	.short	2
	.short	4430
	.short	2
	.short	4430
	.short	.Ltmp781-.Ltmp780
.Ltmp780:
	.short	4429
	.long	0
	.long	0
	.long	4135
	.cv_inline_linetable	14 2 163 .Lfunc_begin3 .Lfunc_end3
	.p2align	2, 0x0
.Ltmp781:
	.short	.Ltmp783-.Ltmp782
.Ltmp782:
	.short	4414
	.long	4097
	.short	1
	.asciz	"self"
	.p2align	2, 0x0
.Ltmp783:
	.cv_def_range	 .Ltmp244 .Ltmp277, reg, 332
	.short	.Ltmp785-.Ltmp784
.Ltmp784:
	.short	4414
	.long	1653
	.short	257
	.asciz	"underflow_bits"
	.p2align	2, 0x0
.Ltmp785:
	.short	.Ltmp787-.Ltmp786
.Ltmp786:
	.short	4414
	.long	32
	.short	257
	.asciz	"bit"
	.p2align	2, 0x0
.Ltmp787:
	.short	.Ltmp789-.Ltmp788
.Ltmp788:
	.short	4429
	.long	0
	.long	0
	.long	4138
	.cv_inline_linetable	15 2 150 .Lfunc_begin3 .Lfunc_end3
	.p2align	2, 0x0
.Ltmp789:
	.short	.Ltmp791-.Ltmp790
.Ltmp790:
	.short	4414
	.long	4097
	.short	1
	.asciz	"self"
	.p2align	2, 0x0
.Ltmp791:
	.cv_def_range	 .Ltmp244 .Ltmp252, reg, 332
	.short	.Ltmp793-.Ltmp792
.Ltmp792:
	.short	4414
	.long	32
	.short	257
	.asciz	"bit"
	.p2align	2, 0x0
.Ltmp793:
	.short	.Ltmp795-.Ltmp794
.Ltmp794:
	.short	4414
	.long	35
	.short	0
	.asciz	"byte_pos"
	.p2align	2, 0x0
.Ltmp795:
	.cv_def_range	 .Ltmp249 .Ltmp252, reg, 331
	.short	.Ltmp797-.Ltmp796
.Ltmp796:
	.short	4414
	.long	32
	.short	256
	.asciz	"bit_pos"
	.p2align	2, 0x0
.Ltmp797:
	.short	2
	.short	4430
	.short	.Ltmp799-.Ltmp798
.Ltmp798:
	.short	4429
	.long	0
	.long	0
	.long	4138
	.cv_inline_linetable	16 2 150 .Lfunc_begin3 .Lfunc_end3
	.p2align	2, 0x0
.Ltmp799:
	.short	.Ltmp801-.Ltmp800
.Ltmp800:
	.short	4414
	.long	4097
	.short	1
	.asciz	"self"
	.p2align	2, 0x0
.Ltmp801:
	.cv_def_range	 .Ltmp255 .Ltmp264 .Ltmp266 .Ltmp277, reg, 332
	.short	.Ltmp803-.Ltmp802
.Ltmp802:
	.short	4414
	.long	32
	.short	257
	.asciz	"bit"
	.p2align	2, 0x0
.Ltmp803:
	.short	.Ltmp805-.Ltmp804
.Ltmp804:
	.short	4414
	.long	35
	.short	0
	.asciz	"byte_pos"
	.p2align	2, 0x0
.Ltmp805:
	.cv_def_range	 .Ltmp259 .Ltmp262, reg, 331
	.cv_def_range	 .Ltmp268 .Ltmp270 .Ltmp275 .Ltmp277, reg, 336
	.short	.Ltmp807-.Ltmp806
.Ltmp806:
	.short	4414
	.long	32
	.short	256
	.asciz	"bit_pos"
	.p2align	2, 0x0
.Ltmp807:
	.short	2
	.short	4430
	.short	2
	.short	4430
	.short	.Ltmp809-.Ltmp808
.Ltmp808:
	.short	4429
	.long	0
	.long	0
	.long	4135
	.cv_inline_linetable	17 2 163 .Lfunc_begin3 .Lfunc_end3
	.p2align	2, 0x0
.Ltmp809:
	.short	.Ltmp811-.Ltmp810
.Ltmp810:
	.short	4414
	.long	4097
	.short	1
	.asciz	"self"
	.p2align	2, 0x0
.Ltmp811:
	.cv_def_range	 .Ltmp297 .Ltmp316 .Ltmp319 .Ltmp332, reg, 332
	.short	.Ltmp813-.Ltmp812
.Ltmp812:
	.short	4414
	.long	1653
	.short	257
	.asciz	"underflow_bits"
	.p2align	2, 0x0
.Ltmp813:
	.short	.Ltmp815-.Ltmp814
.Ltmp814:
	.short	4414
	.long	32
	.short	257
	.asciz	"bit"
	.p2align	2, 0x0
.Ltmp815:
	.short	.Ltmp817-.Ltmp816
.Ltmp816:
	.short	4429
	.long	0
	.long	0
	.long	4138
	.cv_inline_linetable	18 2 150 .Lfunc_begin3 .Lfunc_end3
	.p2align	2, 0x0
.Ltmp817:
	.short	.Ltmp819-.Ltmp818
.Ltmp818:
	.short	4414
	.long	4097
	.short	1
	.asciz	"self"
	.p2align	2, 0x0
.Ltmp819:
	.cv_def_range	 .Ltmp297 .Ltmp304, reg, 332
	.short	.Ltmp821-.Ltmp820
.Ltmp820:
	.short	4414
	.long	32
	.short	257
	.asciz	"bit"
	.p2align	2, 0x0
.Ltmp821:
	.short	.Ltmp823-.Ltmp822
.Ltmp822:
	.short	4414
	.long	35
	.short	0
	.asciz	"byte_pos"
	.p2align	2, 0x0
.Ltmp823:
	.cv_def_range	 .Ltmp301 .Ltmp304, reg, 336
	.short	.Ltmp825-.Ltmp824
.Ltmp824:
	.short	4414
	.long	32
	.short	256
	.asciz	"bit_pos"
	.p2align	2, 0x0
.Ltmp825:
	.short	2
	.short	4430
	.short	.Ltmp827-.Ltmp826
.Ltmp826:
	.short	4429
	.long	0
	.long	0
	.long	4138
	.cv_inline_linetable	22 2 150 .Lfunc_begin3 .Lfunc_end3
	.p2align	2, 0x0
.Ltmp827:
	.short	.Ltmp829-.Ltmp828
.Ltmp828:
	.short	4414
	.long	4097
	.short	1
	.asciz	"self"
	.p2align	2, 0x0
.Ltmp829:
	.cv_def_range	 .Ltmp307 .Ltmp314 .Ltmp321 .Ltmp332, reg, 332
	.short	.Ltmp831-.Ltmp830
.Ltmp830:
	.short	4414
	.long	32
	.short	257
	.asciz	"bit"
	.p2align	2, 0x0
.Ltmp831:
	.short	.Ltmp833-.Ltmp832
.Ltmp832:
	.short	4414
	.long	35
	.short	0
	.asciz	"byte_pos"
	.p2align	2, 0x0
.Ltmp833:
	.cv_def_range	 .Ltmp311 .Ltmp314, reg, 336
	.cv_def_range	 .Ltmp323 .Ltmp325 .Ltmp330 .Ltmp332, reg, 331
	.short	.Ltmp835-.Ltmp834
.Ltmp834:
	.short	4414
	.long	32
	.short	256
	.asciz	"bit_pos"
	.p2align	2, 0x0
.Ltmp835:
	.short	2
	.short	4430
	.short	2
	.short	4430
	.short	.Ltmp837-.Ltmp836
.Ltmp836:
	.short	4429
	.long	0
	.long	0
	.long	4135
	.cv_inline_linetable	19 2 163 .Lfunc_begin3 .Lfunc_end3
	.p2align	2, 0x0
.Ltmp837:
	.short	.Ltmp839-.Ltmp838
.Ltmp838:
	.short	4414
	.long	4097
	.short	1
	.asciz	"self"
	.p2align	2, 0x0
.Ltmp839:
	.cv_def_range	 .Ltmp280 .Ltmp296 .Ltmp334 .Lfunc_end3, reg, 332
	.short	.Ltmp841-.Ltmp840
.Ltmp840:
	.short	4414
	.long	1653
	.short	257
	.asciz	"underflow_bits"
	.p2align	2, 0x0
.Ltmp841:
	.short	.Ltmp843-.Ltmp842
.Ltmp842:
	.short	4414
	.long	32
	.short	257
	.asciz	"bit"
	.p2align	2, 0x0
.Ltmp843:
	.short	.Ltmp845-.Ltmp844
.Ltmp844:
	.short	4429
	.long	0
	.long	0
	.long	4138
	.cv_inline_linetable	20 2 150 .Lfunc_begin3 .Lfunc_end3
	.p2align	2, 0x0
.Ltmp845:
	.short	.Ltmp847-.Ltmp846
.Ltmp846:
	.short	4414
	.long	4097
	.short	1
	.asciz	"self"
	.p2align	2, 0x0
.Ltmp847:
	.cv_def_range	 .Ltmp280 .Ltmp287, reg, 332
	.short	.Ltmp849-.Ltmp848
.Ltmp848:
	.short	4414
	.long	32
	.short	257
	.asciz	"bit"
	.p2align	2, 0x0
.Ltmp849:
	.short	.Ltmp851-.Ltmp850
.Ltmp850:
	.short	4414
	.long	35
	.short	0
	.asciz	"byte_pos"
	.p2align	2, 0x0
.Ltmp851:
	.cv_def_range	 .Ltmp284 .Ltmp287, reg, 336
	.short	.Ltmp853-.Ltmp852
.Ltmp852:
	.short	4414
	.long	32
	.short	256
	.asciz	"bit_pos"
	.p2align	2, 0x0
.Ltmp853:
	.short	2
	.short	4430
	.short	.Ltmp855-.Ltmp854
.Ltmp854:
	.short	4429
	.long	0
	.long	0
	.long	4138
	.cv_inline_linetable	21 2 150 .Lfunc_begin3 .Lfunc_end3
	.p2align	2, 0x0
.Ltmp855:
	.short	.Ltmp857-.Ltmp856
.Ltmp856:
	.short	4414
	.long	4097
	.short	1
	.asciz	"self"
	.p2align	2, 0x0
.Ltmp857:
	.cv_def_range	 .Ltmp288 .Ltmp295 .Ltmp336 .Lfunc_end3, reg, 332
	.short	.Ltmp859-.Ltmp858
.Ltmp858:
	.short	4414
	.long	32
	.short	257
	.asciz	"bit"
	.p2align	2, 0x0
.Ltmp859:
	.short	.Ltmp861-.Ltmp860
.Ltmp860:
	.short	4414
	.long	35
	.short	0
	.asciz	"byte_pos"
	.p2align	2, 0x0
.Ltmp861:
	.cv_def_range	 .Ltmp292 .Ltmp295 .Ltmp338 .Ltmp340 .Ltmp345 .Lfunc_end3, reg, 331
	.short	.Ltmp863-.Ltmp862
.Ltmp862:
	.short	4414
	.long	32
	.short	256
	.asciz	"bit_pos"
	.p2align	2, 0x0
.Ltmp863:
	.short	2
	.short	4430
	.short	2
	.short	4430
	.short	2
	.short	4431
.Ltmp695:
	.p2align	2, 0x0
	.cv_linetable	9, proof.encode, .Lfunc_end3
	.long	241
	.long	.Ltmp865-.Ltmp864
.Ltmp864:
	.short	.Ltmp867-.Ltmp866
.Ltmp866:
	.short	4422
	.long	0
	.long	0
	.long	0
	.long	.Lfunc_end4-proof.RadicalPredictor.observe
	.long	0
	.long	0
	.long	4165
	.secrel32	proof.RadicalPredictor.observe
	.secidx	proof.RadicalPredictor.observe
	.byte	129
	.asciz	"observe"
	.p2align	2, 0x0
.Ltmp867:
	.short	.Ltmp869-.Ltmp868
.Ltmp868:
	.short	4114
	.long	8
	.long	0
	.long	0
	.long	24
	.long	0
	.short	0
	.long	1220608
	.p2align	2, 0x0
.Ltmp869:
	.short	.Ltmp871-.Ltmp870
.Ltmp870:
	.short	4414
	.long	4107
	.short	1
	.asciz	"self"
	.p2align	2, 0x0
.Ltmp871:
	.cv_def_range	 .Lfunc_begin4 .Lfunc_end4, reg, 330
	.short	.Ltmp873-.Ltmp872
.Ltmp872:
	.short	4414
	.long	32
	.short	1
	.asciz	"rc"
	.p2align	2, 0x0
.Ltmp873:
	.cv_def_range	 .Lfunc_begin4 .Lfunc_end4, reg, 3
	.short	.Ltmp875-.Ltmp874
.Ltmp874:
	.short	4414
	.long	32
	.short	1
	.asciz	"rf"
	.p2align	2, 0x0
.Ltmp875:
	.cv_def_range	 .Lfunc_begin4 .Lfunc_end4, reg, 344
	.short	.Ltmp877-.Ltmp876
.Ltmp876:
	.short	4414
	.long	32
	.short	1
	.asciz	"ra"
	.p2align	2, 0x0
.Ltmp877:
	.cv_def_range	 .Lfunc_begin4 .Lfunc_end4, reg, 345
	.short	.Ltmp879-.Ltmp878
.Ltmp878:
	.short	4414
	.long	117
	.short	0
	.asciz	"w"
	.p2align	2, 0x0
.Ltmp879:
	.cv_def_range	 .Ltmp349 .Lfunc_end4, reg, 17
	.short	.Ltmp881-.Ltmp880
.Ltmp880:
	.short	4359
	.long	48
	.byte	0x00, 0x00
	.asciz	"found_rc"
	.p2align	2, 0x0
.Ltmp881:
	.short	.Ltmp883-.Ltmp882
.Ltmp882:
	.short	4414
	.long	117
	.short	0
	.asciz	"key_rc"
	.p2align	2, 0x0
.Ltmp883:
	.cv_def_range	 .Ltmp350 .Ltmp365, reg, 363
	.short	.Ltmp885-.Ltmp884
.Ltmp884:
	.short	4414
	.long	117
	.short	0
	.asciz	"key_rf"
	.p2align	2, 0x0
.Ltmp885:
	.cv_def_range	 .Ltmp366 .Ltmp381, reg, 23
	.short	.Ltmp887-.Ltmp886
.Ltmp886:
	.short	4359
	.long	48
	.byte	0x00, 0x00
	.asciz	"found_rf"
	.p2align	2, 0x0
.Ltmp887:
	.short	.Ltmp889-.Ltmp888
.Ltmp888:
	.short	4414
	.long	117
	.short	0
	.asciz	"key_ra"
	.p2align	2, 0x0
.Ltmp889:
	.cv_def_range	 .Ltmp382 .Lfunc_end4, reg, 363
	.short	.Ltmp891-.Ltmp890
.Ltmp890:
	.short	4359
	.long	48
	.byte	0x00, 0x00
	.asciz	"found_ra"
	.p2align	2, 0x0
.Ltmp891:
	.short	.Ltmp893-.Ltmp892
.Ltmp892:
	.short	4414
	.long	4166
	.short	256
	.asciz	"entry"
	.p2align	2, 0x0
.Ltmp893:
	.short	.Ltmp895-.Ltmp894
.Ltmp894:
	.short	4414
	.long	4166
	.short	256
	.asciz	"entry"
	.p2align	2, 0x0
.Ltmp895:
	.short	.Ltmp897-.Ltmp896
.Ltmp896:
	.short	4414
	.long	4166
	.short	256
	.asciz	"entry"
	.p2align	2, 0x0
.Ltmp897:
	.short	2
	.short	4431
.Ltmp865:
	.p2align	2, 0x0
	.cv_linetable	23, proof.RadicalPredictor.observe, .Lfunc_end4
	.long	241
	.long	.Ltmp899-.Ltmp898
.Ltmp898:
	.short	.Ltmp901-.Ltmp900
.Ltmp900:
	.short	4422
	.long	0
	.long	0
	.long	0
	.long	.Lfunc_end5-proof.RadicalPredictor.getCumFreqsRA
	.long	0
	.long	0
	.long	4170
	.secrel32	proof.RadicalPredictor.getCumFreqsRA
	.secidx	proof.RadicalPredictor.getCumFreqsRA
	.byte	129
	.asciz	"getCumFreqsRA"
	.p2align	2, 0x0
.Ltmp901:
	.short	.Ltmp903-.Ltmp902
.Ltmp902:
	.short	4114
	.long	1032
	.long	0
	.long	0
	.long	16
	.long	0
	.short	0
	.long	1220608
	.p2align	2, 0x0
.Ltmp903:
	.short	.Ltmp905-.Ltmp904
.Ltmp904:
	.short	4414
	.long	4107
	.short	1
	.asciz	"self"
	.p2align	2, 0x0
.Ltmp905:
	.cv_def_range	 .Lfunc_begin5 .Ltmp432, reg, 330
	.short	.Ltmp907-.Ltmp906
.Ltmp906:
	.short	4414
	.long	32
	.short	1
	.asciz	"curr_rc"
	.p2align	2, 0x0
.Ltmp907:
	.cv_def_range	 .Lfunc_begin5 .Ltmp403, reg, 3
	.short	.Ltmp909-.Ltmp908
.Ltmp908:
	.short	4414
	.long	32
	.short	1
	.asciz	"curr_rf"
	.p2align	2, 0x0
.Ltmp909:
	.cv_def_range	 .Lfunc_begin5 .Ltmp404, reg, 344
	.short	.Ltmp911-.Ltmp910
.Ltmp910:
	.short	4414
	.long	32
	.short	1
	.asciz	"prev_ra"
	.p2align	2, 0x0
.Ltmp911:
	.cv_def_range	 .Lfunc_begin5 .Ltmp407 .Ltmp408 .Ltmp409, reg, 345
	.short	.Ltmp913-.Ltmp912
.Ltmp912:
	.short	4414
	.long	4167
	.short	1
	.asciz	"cum_freqs"
	.p2align	2, 0x0
.Ltmp913:
	.cv_def_range	 .Ltmp398 .Lfunc_end5, frame_ptr_rel, 960
	.short	.Ltmp915-.Ltmp914
.Ltmp914:
	.short	4414
	.long	4171
	.short	0
	.asciz	"freqs"
	.p2align	2, 0x0
.Ltmp915:
	.cv_def_range	 .Ltmp399 .Ltmp451, frame_ptr_rel, -128
	.short	.Ltmp917-.Ltmp916
.Ltmp916:
	.short	4414
	.long	117
	.short	0
	.asciz	"key"
	.p2align	2, 0x0
.Ltmp917:
	.cv_def_range	 .Ltmp405 .Ltmp438, reg, 19
	.short	.Ltmp919-.Ltmp918
.Ltmp918:
	.short	4359
	.long	35
	.byte	0x00, 0x00
	.asciz	"i"
	.p2align	2, 0x0
.Ltmp919:
	.short	.Ltmp921-.Ltmp920
.Ltmp920:
	.short	4414
	.long	4116
	.short	0
	.asciz	"entry"
	.p2align	2, 0x0
.Ltmp921:
	.cv_def_range	 .Ltmp417 .Ltmp418 .Ltmp421 .Ltmp422 .Ltmp425 .Ltmp426 .Ltmp429 .Ltmp430, subfield_reg, 24, 4
	.cv_def_range	 .Ltmp437 .Ltmp438, subfield_reg, 363, 4
	.short	2
	.short	4431
.Ltmp899:
	.p2align	2, 0x0
	.cv_linetable	24, proof.RadicalPredictor.getCumFreqsRA, .Lfunc_end5
	.long	241
	.long	.Ltmp923-.Ltmp922
.Ltmp922:
	.short	.Ltmp925-.Ltmp924
.Ltmp924:
	.short	4422
	.long	0
	.long	0
	.long	0
	.long	.Lfunc_end6-proof.RadicalPredictor.getCumFreqsRF
	.long	0
	.long	0
	.long	4174
	.secrel32	proof.RadicalPredictor.getCumFreqsRF
	.secidx	proof.RadicalPredictor.getCumFreqsRF
	.byte	129
	.asciz	"getCumFreqsRF"
	.p2align	2, 0x0
.Ltmp925:
	.short	.Ltmp927-.Ltmp926
.Ltmp926:
	.short	4114
	.long	1032
	.long	0
	.long	0
	.long	16
	.long	0
	.short	0
	.long	1220608
	.p2align	2, 0x0
.Ltmp927:
	.short	.Ltmp929-.Ltmp928
.Ltmp928:
	.short	4414
	.long	4107
	.short	1
	.asciz	"self"
	.p2align	2, 0x0
.Ltmp929:
	.cv_def_range	 .Lfunc_begin6 .Ltmp484, reg, 330
	.short	.Ltmp931-.Ltmp930
.Ltmp930:
	.short	4414
	.long	32
	.short	1
	.asciz	"curr_rc"
	.p2align	2, 0x0
.Ltmp931:
	.cv_def_range	 .Lfunc_begin6 .Ltmp456, reg, 3
	.short	.Ltmp933-.Ltmp932
.Ltmp932:
	.short	4414
	.long	32
	.short	1
	.asciz	"prev_rf"
	.p2align	2, 0x0
.Ltmp933:
	.cv_def_range	 .Lfunc_begin6 .Ltmp459 .Ltmp460 .Ltmp461, reg, 344
	.short	.Ltmp935-.Ltmp934
.Ltmp934:
	.short	4414
	.long	4167
	.short	1
	.asciz	"cum_freqs"
	.p2align	2, 0x0
.Ltmp935:
	.cv_def_range	 .Lfunc_begin6 .Lfunc_end6, reg, 337
	.short	.Ltmp937-.Ltmp936
.Ltmp936:
	.short	4414
	.long	4171
	.short	0
	.asciz	"freqs"
	.p2align	2, 0x0
.Ltmp937:
	.cv_def_range	 .Ltmp452 .Ltmp503, frame_ptr_rel, -128
	.short	.Ltmp939-.Ltmp938
.Ltmp938:
	.short	4414
	.long	117
	.short	0
	.asciz	"key"
	.p2align	2, 0x0
.Ltmp939:
	.cv_def_range	 .Ltmp457 .Ltmp490, reg, 17
	.short	.Ltmp941-.Ltmp940
.Ltmp940:
	.short	4359
	.long	35
	.byte	0x00, 0x00
	.asciz	"i"
	.p2align	2, 0x0
.Ltmp941:
	.short	.Ltmp943-.Ltmp942
.Ltmp942:
	.short	4414
	.long	4116
	.short	0
	.asciz	"entry"
	.p2align	2, 0x0
.Ltmp943:
	.cv_def_range	 .Ltmp469 .Ltmp470 .Ltmp473 .Ltmp474 .Ltmp477 .Ltmp478 .Ltmp481 .Ltmp482, subfield_reg, 24, 4
	.cv_def_range	 .Ltmp489 .Ltmp490, subfield_reg, 363, 4
	.short	2
	.short	4431
.Ltmp923:
	.p2align	2, 0x0
	.cv_linetable	25, proof.RadicalPredictor.getCumFreqsRF, .Lfunc_end6
	.long	241
	.long	.Ltmp945-.Ltmp944
.Ltmp944:
	.short	.Ltmp947-.Ltmp946
.Ltmp946:
	.short	4422
	.long	0
	.long	0
	.long	0
	.long	.Lfunc_end7-proof.RadicalPredictor.getCumFreqsRC
	.long	0
	.long	0
	.long	4177
	.secrel32	proof.RadicalPredictor.getCumFreqsRC
	.secidx	proof.RadicalPredictor.getCumFreqsRC
	.byte	129
	.asciz	"getCumFreqsRC"
	.p2align	2, 0x0
.Ltmp947:
	.short	.Ltmp949-.Ltmp948
.Ltmp948:
	.short	4114
	.long	1032
	.long	0
	.long	0
	.long	16
	.long	0
	.short	0
	.long	1220608
	.p2align	2, 0x0
.Ltmp949:
	.short	.Ltmp951-.Ltmp950
.Ltmp950:
	.short	4414
	.long	4107
	.short	1
	.asciz	"self"
	.p2align	2, 0x0
.Ltmp951:
	.cv_def_range	 .Lfunc_begin7 .Ltmp533, reg, 330
	.short	.Ltmp953-.Ltmp952
.Ltmp952:
	.short	4414
	.long	32
	.short	1
	.asciz	"prev_rc"
	.p2align	2, 0x0
.Ltmp953:
	.cv_def_range	 .Lfunc_begin7 .Ltmp507, reg, 3
	.short	.Ltmp955-.Ltmp954
.Ltmp954:
	.short	4414
	.long	4167
	.short	1
	.asciz	"cum_freqs"
	.p2align	2, 0x0
.Ltmp955:
	.cv_def_range	 .Lfunc_begin7 .Lfunc_end7, reg, 336
	.short	.Ltmp957-.Ltmp956
.Ltmp956:
	.short	4414
	.long	4171
	.short	0
	.asciz	"freqs"
	.p2align	2, 0x0
.Ltmp957:
	.cv_def_range	 .Ltmp504 .Ltmp552, frame_ptr_rel, -128
	.short	.Ltmp959-.Ltmp958
.Ltmp958:
	.short	4359
	.long	35
	.byte	0x00, 0x00
	.asciz	"i"
	.p2align	2, 0x0
.Ltmp959:
	.short	.Ltmp961-.Ltmp960
.Ltmp960:
	.short	4414
	.long	4116
	.short	0
	.asciz	"entry"
	.p2align	2, 0x0
.Ltmp961:
	.cv_def_range	 .Ltmp518 .Ltmp519 .Ltmp522 .Ltmp523 .Ltmp526 .Ltmp527 .Ltmp530 .Ltmp531, subfield_reg, 24, 4
	.cv_def_range	 .Ltmp538 .Ltmp539, subfield_reg, 363, 4
	.short	2
	.short	4431
.Ltmp945:
	.p2align	2, 0x0
	.cv_linetable	26, proof.RadicalPredictor.getCumFreqsRC, .Lfunc_end7
	.long	241
	.long	.Ltmp963-.Ltmp962
.Ltmp962:
	.short	.Ltmp965-.Ltmp964
.Ltmp964:
	.short	4422
	.long	0
	.long	0
	.long	0
	.long	.Lfunc_end8-proof.wasm_decode
	.long	0
	.long	0
	.long	4180
	.secrel32	proof.wasm_decode
	.secidx	proof.wasm_decode
	.byte	129
	.asciz	"wasm_decode"
	.p2align	2, 0x0
.Ltmp965:
	.short	.Ltmp967-.Ltmp966
.Ltmp966:
	.short	4114
	.long	48
	.long	0
	.long	0
	.long	8
	.long	0
	.short	0
	.long	1220608
	.p2align	2, 0x0
.Ltmp967:
	.short	.Ltmp969-.Ltmp968
.Ltmp968:
	.short	4414
	.long	1568
	.short	1
	.asciz	"encoded_ptr"
	.p2align	2, 0x0
.Ltmp969:
	.cv_def_range	 .Lfunc_begin8 .Ltmp554, reg, 330
	.short	.Ltmp971-.Ltmp970
.Ltmp970:
	.short	4414
	.long	35
	.short	1
	.asciz	"encoded_len"
	.p2align	2, 0x0
.Ltmp971:
	.cv_def_range	 .Lfunc_begin8 .Ltmp554, reg, 331
	.short	.Ltmp973-.Ltmp972
.Ltmp972:
	.short	4414
	.long	35
	.short	1
	.asciz	"count"
	.p2align	2, 0x0
.Ltmp973:
	.cv_def_range	 .Lfunc_begin8 .Ltmp554, reg, 336
	.short	2
	.short	4431
.Ltmp963:
	.p2align	2, 0x0
	.cv_linetable	27, proof.wasm_decode, .Lfunc_end8
	.long	241
	.long	.Ltmp975-.Ltmp974
.Ltmp974:
	.short	.Ltmp977-.Ltmp976
.Ltmp976:
	.short	4422
	.long	0
	.long	0
	.long	0
	.long	.Lfunc_end9-proof.wasm_get_encoded_bits
	.long	0
	.long	0
	.long	4182
	.secrel32	proof.wasm_get_encoded_bits
	.secidx	proof.wasm_get_encoded_bits
	.byte	129
	.asciz	"wasm_get_encoded_bits"
	.p2align	2, 0x0
.Ltmp977:
	.short	.Ltmp979-.Ltmp978
.Ltmp978:
	.short	4114
	.long	8
	.long	0
	.long	0
	.long	0
	.long	0
	.short	0
	.long	1220608
	.p2align	2, 0x0
.Ltmp979:
	.short	2
	.short	4431
.Ltmp975:
	.p2align	2, 0x0
	.cv_linetable	28, proof.wasm_get_encoded_bits, .Lfunc_end9
	.long	241
	.long	.Ltmp981-.Ltmp980
.Ltmp980:
	.short	.Ltmp983-.Ltmp982
.Ltmp982:
	.short	4422
	.long	0
	.long	0
	.long	0
	.long	.Lfunc_end10-proof.wasm_encode
	.long	0
	.long	0
	.long	4185
	.secrel32	proof.wasm_encode
	.secidx	proof.wasm_encode
	.byte	129
	.asciz	"wasm_encode"
	.p2align	2, 0x0
.Ltmp983:
	.short	.Ltmp985-.Ltmp984
.Ltmp984:
	.short	4114
	.long	48
	.long	0
	.long	0
	.long	24
	.long	0
	.short	0
	.long	1220608
	.p2align	2, 0x0
.Ltmp985:
	.short	.Ltmp987-.Ltmp986
.Ltmp986:
	.short	4414
	.long	4152
	.short	1
	.asciz	"concepts_ptr"
	.p2align	2, 0x0
.Ltmp987:
	.cv_def_range	 .Lfunc_begin10 .Ltmp559, reg, 330
	.cv_def_range	 .Ltmp559 .Ltmp561, reg, 333
	.short	.Ltmp989-.Ltmp988
.Ltmp988:
	.short	4414
	.long	35
	.short	1
	.asciz	"count"
	.p2align	2, 0x0
.Ltmp989:
	.cv_def_range	 .Lfunc_begin10 .Ltmp560, reg, 331
	.cv_def_range	 .Ltmp560 .Ltmp562, reg, 332
	.short	2
	.short	4431
.Ltmp981:
	.p2align	2, 0x0
	.cv_linetable	29, proof.wasm_encode, .Lfunc_end10
	.long	241
	.long	.Ltmp991-.Ltmp990
.Ltmp990:
	.short	.Ltmp993-.Ltmp992
.Ltmp992:
	.short	4364
	.long	4193
	.secrel32	os.windows.tls._tls_used
	.secidx	os.windows.tls._tls_used
	.asciz	"_tls_used"
	.p2align	2, 0x0
.Ltmp993:
	.short	.Ltmp995-.Ltmp994
.Ltmp994:
	.short	4364
	.long	4190
	.secrel32	os.windows.tls.__xl_z
	.secidx	os.windows.tls.__xl_z
	.asciz	"__xl_z"
	.p2align	2, 0x0
.Ltmp995:
	.short	.Ltmp997-.Ltmp996
.Ltmp996:
	.short	4364
	.long	1536
	.secrel32	os.windows.tls._tls_end
	.secidx	os.windows.tls._tls_end
	.asciz	"_tls_end"
	.p2align	2, 0x0
.Ltmp997:
	.short	.Ltmp999-.Ltmp998
.Ltmp998:
	.short	4364
	.long	1536
	.secrel32	os.windows.tls._tls_start
	.secidx	os.windows.tls._tls_start
	.asciz	"_tls_start"
	.p2align	2, 0x0
.Ltmp999:
	.short	.Ltmp1001-.Ltmp1000
.Ltmp1000:
	.short	4364
	.long	4190
	.secrel32	os.windows.tls.__xl_a
	.secidx	os.windows.tls.__xl_a
	.asciz	"__xl_a"
	.p2align	2, 0x0
.Ltmp1001:
	.short	.Ltmp1003-.Ltmp1002
.Ltmp1002:
	.short	4364
	.long	117
	.secrel32	os.windows.tls._tls_index
	.secidx	os.windows.tls._tls_index
	.asciz	"_tls_index"
	.p2align	2, 0x0
.Ltmp1003:
	.short	.Ltmp1005-.Ltmp1004
.Ltmp1004:
	.short	4364
	.long	4102
	.secrel32	proof.global_writer
	.secidx	proof.global_writer
	.asciz	"global_writer"
	.p2align	2, 0x0
.Ltmp1005:
	.short	.Ltmp1007-.Ltmp1006
.Ltmp1006:
	.short	4364
	.long	4196
	.secrel32	proof.global_decoded_buf
	.secidx	proof.global_decoded_buf
	.asciz	"global_decoded_buf"
	.p2align	2, 0x0
.Ltmp1007:
.Ltmp991:
	.p2align	2, 0x0
	.long	241
	.long	.Ltmp1009-.Ltmp1008
.Ltmp1008:
	.short	.Ltmp1011-.Ltmp1010
.Ltmp1010:
	.short	4360
	.long	4102
	.asciz	"proof.BitWriter"
	.p2align	2, 0x0
.Ltmp1011:
	.short	.Ltmp1013-.Ltmp1012
.Ltmp1012:
	.short	4360
	.long	4113
	.asciz	"proof.RadicalPredictor"
	.p2align	2, 0x0
.Ltmp1013:
	.short	.Ltmp1015-.Ltmp1014
.Ltmp1014:
	.short	4360
	.long	4116
	.asciz	"proof.SparseTransition"
	.p2align	2, 0x0
.Ltmp1015:
	.short	.Ltmp1017-.Ltmp1016
.Ltmp1016:
	.short	4360
	.long	4125
	.asciz	"proof.BitReader"
	.p2align	2, 0x0
.Ltmp1017:
	.short	.Ltmp1019-.Ltmp1018
.Ltmp1018:
	.short	4360
	.long	4128
	.asciz	"[]const u8"
	.p2align	2, 0x0
.Ltmp1019:
	.short	.Ltmp1021-.Ltmp1020
.Ltmp1020:
	.short	4360
	.long	4147
	.asciz	"proof.Concept6D"
	.p2align	2, 0x0
.Ltmp1021:
	.short	.Ltmp1023-.Ltmp1022
.Ltmp1022:
	.short	4360
	.long	4154
	.asciz	"[]proof.Concept6D"
	.p2align	2, 0x0
.Ltmp1023:
	.short	.Ltmp1025-.Ltmp1024
.Ltmp1024:
	.short	4360
	.long	4160
	.asciz	"[]const proof.Concept6D"
	.p2align	2, 0x0
.Ltmp1025:
	.short	.Ltmp1027-.Ltmp1026
.Ltmp1026:
	.short	4360
	.long	1536
	.asciz	"?*anyopaque"
	.p2align	2, 0x0
.Ltmp1027:
	.short	.Ltmp1029-.Ltmp1028
.Ltmp1028:
	.short	4360
	.long	4190
	.asciz	"?*const fn (*anyopaque, u32, *anyopaque) callconv(.c) void"
	.p2align	2, 0x0
.Ltmp1029:
	.short	.Ltmp1031-.Ltmp1030
.Ltmp1030:
	.short	4360
	.long	4193
	.asciz	"os.windows.tls.IMAGE_TLS_DIRECTORY"
	.p2align	2, 0x0
.Ltmp1031:
.Ltmp1009:
	.p2align	2, 0x0
	.cv_filechecksums
	.cv_stringtable
	.long	241
	.long	.Ltmp1033-.Ltmp1032
.Ltmp1032:
	.short	.Ltmp1035-.Ltmp1034
.Ltmp1034:
	.short	4428
	.long	4200
	.p2align	2, 0x0
.Ltmp1035:
.Ltmp1033:
	.p2align	2, 0x0
	.section	.debug$T,"dr"
	.p2align	2, 0x0
	.long	4
	.short	0x26
	.short	0x1505
	.short	0x0
	.short	0x80
	.long	0x0
	.long	0x0
	.long	0x0
	.short	0x0
	.asciz	"proof.BitWriter"
	.byte	242
	.byte	241
	.short	0xa
	.short	0x1002
	.long	0x1000
	.long	0x1000c
	.short	0xa
	.short	0x1201
	.long	0x1
	.long	0x1001
	.short	0xe
	.short	0x1008
	.long	0x0
	.byte	0x0
	.byte	0x0
	.short	0x1
	.long	0x1002
	.short	0x16
	.short	0x1503
	.long	0x20
	.long	0x23
	.short	0x2800
	.asciz	"[10240]u8"
	.short	0x2a
	.short	0x1203
	.short	0x150d
	.short	0x3
	.long	0x23
	.short	0x0
	.asciz	"bit_index"
	.short	0x150d
	.short	0x3
	.long	0x1004
	.short	0x8
	.asciz	"buffer"
	.byte	243
	.byte	242
	.byte	241
	.short	0x26
	.short	0x1505
	.short	0x2
	.short	0x0
	.long	0x1005
	.long	0x0
	.long	0x0
	.short	0x2808
	.asciz	"proof.BitWriter"
	.byte	242
	.byte	241
	.short	0x3a
	.short	0x1605
	.long	0x0
	.asciz	"J:\\Language-U\\WASM_U-Performance_Record\\proof.zig"
	.byte	242
	.byte	241
	.short	0xe
	.short	0x1606
	.long	0x1006
	.long	0x1007
	.long	0xa9
	.short	0x12
	.short	0x1601
	.long	0x0
	.long	0x1003
	.asciz	"init"
	.byte	243
	.byte	242
	.byte	241
	.short	0x2e
	.short	0x1505
	.short	0x0
	.short	0x80
	.long	0x0
	.long	0x0
	.long	0x0
	.short	0x0
	.asciz	"proof.RadicalPredictor"
	.byte	243
	.byte	242
	.byte	241
	.short	0xa
	.short	0x1002
	.long	0x100a
	.long	0x1000c
	.short	0x12
	.short	0x1201
	.long	0x3
	.long	0x100b
	.long	0x75
	.long	0x75
	.short	0xe
	.short	0x1008
	.long	0x0
	.byte	0x0
	.byte	0x0
	.short	0x3
	.long	0x100c
	.short	0x2e
	.short	0x1505
	.short	0x0
	.short	0x80
	.long	0x0
	.long	0x0
	.long	0x0
	.short	0x0
	.asciz	"proof.SparseTransition"
	.byte	243
	.byte	242
	.byte	241
	.short	0x2a
	.short	0x1503
	.long	0x100e
	.long	0x23
	.short	0xc00
	.asciz	"[256]proof.SparseTransition"
	.byte	242
	.byte	241
	.short	0xe6
	.short	0x1203
	.short	0x150d
	.short	0x3
	.long	0x23
	.short	0x0
	.asciz	"trans_rc_len"
	.byte	241
	.short	0x150d
	.short	0x3
	.long	0x23
	.short	0x8
	.asciz	"trans_rf_len"
	.byte	241
	.short	0x150d
	.short	0x3
	.long	0x23
	.short	0x10
	.asciz	"trans_ra_len"
	.byte	241
	.short	0x150d
	.short	0x3
	.long	0x75
	.short	0x18
	.asciz	"alpha"
	.short	0x150d
	.short	0x3
	.long	0x75
	.short	0x1c
	.asciz	"weight"
	.byte	243
	.byte	242
	.byte	241
	.short	0x150d
	.short	0x3
	.long	0x100f
	.short	0x20
	.asciz	"trans_rc"
	.byte	241
	.short	0x150d
	.short	0x3
	.long	0x100f
	.short	0xc20
	.asciz	"trans_rf"
	.byte	241
	.short	0x150d
	.short	0x3
	.long	0x100f
	.short	0x1820
	.asciz	"trans_ra"
	.byte	241
	.short	0x150d
	.short	0x3
	.long	0x20
	.short	0x2420
	.asciz	"prev_rc"
	.byte	242
	.byte	241
	.short	0x150d
	.short	0x3
	.long	0x20
	.short	0x2421
	.asciz	"prev_rf"
	.byte	242
	.byte	241
	.short	0x150d
	.short	0x3
	.long	0x20
	.short	0x2422
	.asciz	"prev_ra"
	.byte	242
	.byte	241
	.short	0x2e
	.short	0x1505
	.short	0xb
	.short	0x0
	.long	0x1010
	.long	0x0
	.long	0x0
	.short	0x2428
	.asciz	"proof.RadicalPredictor"
	.byte	243
	.byte	242
	.byte	241
	.short	0xe
	.short	0x1606
	.long	0x1011
	.long	0x1007
	.long	0x88
	.short	0x32
	.short	0x1203
	.short	0x150d
	.short	0x3
	.long	0x75
	.short	0x0
	.asciz	"key"
	.byte	242
	.byte	241
	.short	0x150d
	.short	0x3
	.long	0x75
	.short	0x4
	.asciz	"count"
	.short	0x150d
	.short	0x3
	.long	0x20
	.short	0x8
	.asciz	"sym"
	.byte	242
	.byte	241
	.short	0x2e
	.short	0x1505
	.short	0x3
	.short	0x0
	.long	0x1013
	.long	0x0
	.long	0x0
	.short	0xc
	.asciz	"proof.SparseTransition"
	.byte	243
	.byte	242
	.byte	241
	.short	0xe
	.short	0x1606
	.long	0x1014
	.long	0x1007
	.long	0x6
	.short	0x12
	.short	0x1601
	.long	0x0
	.long	0x100d
	.asciz	"init"
	.byte	243
	.byte	242
	.byte	241
	.short	0x26
	.short	0x1505
	.short	0x0
	.short	0x80
	.long	0x0
	.long	0x0
	.long	0x0
	.short	0x0
	.asciz	"proof.BitReader"
	.byte	242
	.byte	241
	.short	0xa
	.short	0x1002
	.long	0x1017
	.long	0x1000c
	.short	0x22
	.short	0x1505
	.short	0x0
	.short	0x80
	.long	0x0
	.long	0x0
	.long	0x0
	.short	0x0
	.asciz	"[]const u8"
	.byte	243
	.byte	242
	.byte	241
	.short	0xe
	.short	0x1201
	.long	0x2
	.long	0x1018
	.long	0x1019
	.short	0xe
	.short	0x1008
	.long	0x0
	.byte	0x0
	.byte	0x0
	.short	0x2
	.long	0x101a
	.short	0x42
	.short	0x1203
	.short	0x150d
	.short	0x3
	.long	0x1019
	.short	0x0
	.asciz	"buffer"
	.byte	243
	.byte	242
	.byte	241
	.short	0x150d
	.short	0x3
	.long	0x23
	.short	0x10
	.asciz	"bit_index"
	.short	0x150d
	.short	0x3
	.long	0x23
	.short	0x18
	.asciz	"total_bits"
	.byte	243
	.byte	242
	.byte	241
	.short	0x26
	.short	0x1505
	.short	0x3
	.short	0x0
	.long	0x101c
	.long	0x0
	.long	0x0
	.short	0x20
	.asciz	"proof.BitReader"
	.byte	242
	.byte	241
	.short	0xe
	.short	0x1606
	.long	0x101d
	.long	0x1007
	.long	0xc2
	.short	0x22
	.short	0x1203
	.short	0x150d
	.short	0x3
	.long	0x620
	.short	0x0
	.asciz	"ptr"
	.byte	242
	.byte	241
	.short	0x150d
	.short	0x3
	.long	0x23
	.short	0x8
	.asciz	"len"
	.byte	242
	.byte	241
	.short	0x22
	.short	0x1505
	.short	0x2
	.short	0x0
	.long	0x101f
	.long	0x0
	.long	0x0
	.short	0x10
	.asciz	"[]const u8"
	.byte	243
	.byte	242
	.byte	241
	.short	0x12
	.short	0x1601
	.long	0x0
	.long	0x101b
	.asciz	"init"
	.byte	243
	.byte	242
	.byte	241
	.short	0xa
	.short	0x1201
	.long	0x1
	.long	0x1018
	.short	0xe
	.short	0x1008
	.long	0x20
	.byte	0x0
	.byte	0x0
	.short	0x1
	.long	0x1022
	.short	0x12
	.short	0x1601
	.long	0x0
	.long	0x1023
	.asciz	"readBit"
	.short	0x12
	.short	0x1201
	.long	0x3
	.long	0x1001
	.long	0x675
	.long	0x20
	.short	0xe
	.short	0x1008
	.long	0x0
	.byte	0x0
	.byte	0x0
	.short	0x3
	.long	0x1025
	.short	0x1a
	.short	0x1601
	.long	0x0
	.long	0x1026
	.asciz	"writeBitHelper"
	.byte	241
	.short	0xe
	.short	0x1201
	.long	0x2
	.long	0x1001
	.long	0x20
	.short	0xe
	.short	0x1008
	.long	0x0
	.byte	0x0
	.byte	0x0
	.short	0x2
	.long	0x1028
	.short	0x16
	.short	0x1601
	.long	0x0
	.long	0x1029
	.asciz	"writeBit"
	.byte	243
	.byte	242
	.byte	241
	.short	0x6
	.short	0x1201
	.long	0x0
	.short	0xe
	.short	0x1008
	.long	0x0
	.byte	0x0
	.byte	0x0
	.short	0x0
	.long	0x102b
	.short	0x16
	.short	0x1601
	.long	0x0
	.long	0x102c
	.asciz	"WinStartup"
	.byte	241
	.short	0xe
	.short	0x1008
	.long	0x74
	.byte	0x0
	.byte	0x0
	.short	0x0
	.long	0x102b
	.short	0x1e
	.short	0x1601
	.long	0x0
	.long	0x102e
	.asciz	"run_verification"
	.byte	243
	.byte	242
	.byte	241
	.short	0x26
	.short	0x1505
	.short	0x0
	.short	0x80
	.long	0x0
	.long	0x0
	.long	0x0
	.short	0x0
	.asciz	"proof.Concept6D"
	.byte	242
	.byte	241
	.short	0x22
	.short	0x1503
	.long	0x1030
	.long	0x23
	.short	0x1e
	.asciz	"[5]proof.Concept6D"
	.byte	243
	.byte	242
	.byte	241
	.short	0x76
	.short	0x1203
	.short	0x150d
	.short	0x3
	.long	0x20
	.short	0x0
	.asciz	"domain"
	.byte	243
	.byte	242
	.byte	241
	.short	0x150d
	.short	0x3
	.long	0x20
	.short	0x1
	.asciz	"subdomain"
	.short	0x150d
	.short	0x3
	.long	0x20
	.short	0x2
	.asciz	"operation"
	.short	0x150d
	.short	0x3
	.long	0x20
	.short	0x3
	.asciz	"modality"
	.byte	241
	.short	0x150d
	.short	0x3
	.long	0x20
	.short	0x4
	.asciz	"depth"
	.short	0x150d
	.short	0x3
	.long	0x20
	.short	0x5
	.asciz	"polarity"
	.byte	241
	.short	0x26
	.short	0x1505
	.short	0x6
	.short	0x0
	.long	0x1032
	.long	0x0
	.long	0x0
	.short	0x6
	.asciz	"proof.Concept6D"
	.byte	242
	.byte	241
	.short	0xe
	.short	0x1606
	.long	0x1033
	.long	0x1007
	.long	0xc5
	.short	0x26
	.short	0x1505
	.short	0x0
	.short	0x80
	.long	0x0
	.long	0x0
	.long	0x0
	.short	0x0
	.asciz	"[]proof.Concept6D"
	.short	0x12
	.short	0x1201
	.long	0x3
	.long	0x1019
	.long	0x23
	.long	0x1035
	.short	0xe
	.short	0x1008
	.long	0x0
	.byte	0x0
	.byte	0x0
	.short	0x3
	.long	0x1036
	.short	0xa
	.short	0x1002
	.long	0x1030
	.long	0x1000c
	.short	0x22
	.short	0x1203
	.short	0x150d
	.short	0x3
	.long	0x1038
	.short	0x0
	.asciz	"ptr"
	.byte	242
	.byte	241
	.short	0x150d
	.short	0x3
	.long	0x23
	.short	0x8
	.asciz	"len"
	.byte	242
	.byte	241
	.short	0x26
	.short	0x1505
	.short	0x2
	.short	0x0
	.long	0x1039
	.long	0x0
	.long	0x0
	.short	0x10
	.asciz	"[]proof.Concept6D"
	.short	0x12
	.short	0x1601
	.long	0x0
	.long	0x1037
	.asciz	"decode"
	.byte	241
	.short	0x12
	.short	0x1503
	.long	0x20
	.long	0x23
	.short	0x3
	.asciz	"[3]u8"
	.short	0x2e
	.short	0x1505
	.short	0x0
	.short	0x80
	.long	0x0
	.long	0x0
	.long	0x0
	.short	0x0
	.asciz	"[]const proof.Concept6D"
	.byte	242
	.byte	241
	.short	0xe
	.short	0x1201
	.long	0x2
	.long	0x103d
	.long	0x1001
	.short	0xe
	.short	0x1008
	.long	0x0
	.byte	0x0
	.byte	0x0
	.short	0x2
	.long	0x103e
	.short	0x2e
	.short	0x1505
	.short	0x2
	.short	0x0
	.long	0x1039
	.long	0x0
	.long	0x0
	.short	0x10
	.asciz	"[]const proof.Concept6D"
	.byte	242
	.byte	241
	.short	0x12
	.short	0x1601
	.long	0x0
	.long	0x103f
	.asciz	"encode"
	.byte	241
	.short	0x16
	.short	0x1503
	.long	0x75
	.long	0x23
	.short	0x404
	.asciz	"[257]u32"
	.byte	241
	.short	0x16
	.short	0x1201
	.long	0x4
	.long	0x100b
	.long	0x20
	.long	0x20
	.long	0x20
	.short	0xe
	.short	0x1008
	.long	0x0
	.byte	0x0
	.byte	0x0
	.short	0x4
	.long	0x1043
	.short	0x12
	.short	0x1601
	.long	0x0
	.long	0x1044
	.asciz	"observe"
	.short	0xa
	.short	0x1002
	.long	0x100e
	.long	0x1000c
	.short	0xa
	.short	0x1002
	.long	0x1042
	.long	0x1000c
	.short	0x1a
	.short	0x1201
	.long	0x5
	.long	0x100b
	.long	0x20
	.long	0x20
	.long	0x20
	.long	0x1047
	.short	0xe
	.short	0x1008
	.long	0x0
	.byte	0x0
	.byte	0x0
	.short	0x5
	.long	0x1048
	.short	0x1a
	.short	0x1601
	.long	0x0
	.long	0x1049
	.asciz	"getCumFreqsRA"
	.byte	242
	.byte	241
	.short	0x16
	.short	0x1503
	.long	0x75
	.long	0x23
	.short	0x400
	.asciz	"[256]u32"
	.byte	241
	.short	0x16
	.short	0x1201
	.long	0x4
	.long	0x100b
	.long	0x20
	.long	0x20
	.long	0x1047
	.short	0xe
	.short	0x1008
	.long	0x0
	.byte	0x0
	.byte	0x0
	.short	0x4
	.long	0x104c
	.short	0x1a
	.short	0x1601
	.long	0x0
	.long	0x104d
	.asciz	"getCumFreqsRF"
	.byte	242
	.byte	241
	.short	0x12
	.short	0x1201
	.long	0x3
	.long	0x100b
	.long	0x20
	.long	0x1047
	.short	0xe
	.short	0x1008
	.long	0x0
	.byte	0x0
	.byte	0x0
	.short	0x3
	.long	0x104f
	.short	0x1a
	.short	0x1601
	.long	0x0
	.long	0x1050
	.asciz	"getCumFreqsRC"
	.byte	242
	.byte	241
	.short	0x12
	.short	0x1201
	.long	0x3
	.long	0x620
	.long	0x23
	.long	0x23
	.short	0xe
	.short	0x1008
	.long	0x1038
	.byte	0x0
	.byte	0x0
	.short	0x3
	.long	0x1052
	.short	0x16
	.short	0x1601
	.long	0x0
	.long	0x1053
	.asciz	"wasm_decode"
	.short	0xe
	.short	0x1008
	.long	0x23
	.byte	0x0
	.byte	0x0
	.short	0x0
	.long	0x102b
	.short	0x22
	.short	0x1601
	.long	0x0
	.long	0x1055
	.asciz	"wasm_get_encoded_bits"
	.byte	242
	.byte	241
	.short	0xe
	.short	0x1201
	.long	0x2
	.long	0x1038
	.long	0x23
	.short	0xe
	.short	0x1008
	.long	0x620
	.byte	0x0
	.byte	0x0
	.short	0x2
	.long	0x1057
	.short	0x16
	.short	0x1601
	.long	0x0
	.long	0x1058
	.asciz	"wasm_encode"
	.short	0x3a
	.short	0x1505
	.short	0x0
	.short	0x80
	.long	0x0
	.long	0x0
	.long	0x0
	.short	0x0
	.asciz	"os.windows.tls.IMAGE_TLS_DIRECTORY"
	.byte	243
	.byte	242
	.byte	241
	.short	0xa
	.short	0x1002
	.long	0x600
	.long	0x1000c
	.short	0x12
	.short	0x1201
	.long	0x3
	.long	0x600
	.long	0x75
	.long	0x600
	.short	0xe
	.short	0x1008
	.long	0x0
	.byte	0x0
	.byte	0x0
	.short	0x3
	.long	0x105c
	.short	0xa
	.short	0x1002
	.long	0x105d
	.long	0x1000c
	.short	0xa
	.short	0x1002
	.long	0x105e
	.long	0x1000c
	.short	0xb6
	.short	0x1203
	.short	0x150d
	.short	0x3
	.long	0x105b
	.short	0x0
	.asciz	"StartAddressOfRawData"
	.short	0x150d
	.short	0x3
	.long	0x105b
	.short	0x8
	.asciz	"EndAddressOfRawData"
	.byte	242
	.byte	241
	.short	0x150d
	.short	0x3
	.long	0x675
	.short	0x10
	.asciz	"AddressOfIndex"
	.byte	243
	.byte	242
	.byte	241
	.short	0x150d
	.short	0x3
	.long	0x105f
	.short	0x18
	.asciz	"AddressOfCallBacks"
	.byte	243
	.byte	242
	.byte	241
	.short	0x150d
	.short	0x3
	.long	0x75
	.short	0x20
	.asciz	"SizeOfZeroFill"
	.byte	243
	.byte	242
	.byte	241
	.short	0x150d
	.short	0x3
	.long	0x75
	.short	0x24
	.asciz	"Characteristics"
	.byte	242
	.byte	241
	.short	0x3a
	.short	0x1505
	.short	0x6
	.short	0x0
	.long	0x1060
	.long	0x0
	.long	0x0
	.short	0x28
	.asciz	"os.windows.tls.IMAGE_TLS_DIRECTORY"
	.byte	243
	.byte	242
	.byte	241
	.short	0xa2
	.short	0x1605
	.long	0x0
	.asciz	"C:\\Users\\freed\\AppData\\Local\\Microsoft\\WinGet\\Packages\\zig.zig_Microsoft.Winget.Source_8wekyb3d8bbwe\\zig-x86_64-windows-0.16.0\\lib\\std\\os\\windows\\tls.zig"
	.byte	242
	.byte	241
	.short	0xe
	.short	0x1606
	.long	0x1061
	.long	0x1062
	.long	0x20
	.short	0x22
	.short	0x1503
	.long	0x1030
	.long	0x23
	.short	0x1770
	.asciz	"[1000]proof.Concept6D"
	.short	0x2e
	.short	0x1605
	.long	0x0
	.asciz	"J:\\Language-U\\WASM_U-Performance_Record"
	.short	0xe
	.short	0x1605
	.long	0x0
	.asciz	"proof"
	.byte	242
	.byte	241
	.short	0xa
	.short	0x1605
	.long	0x0
	.byte	0
	.byte	243
	.byte	242
	.byte	241
	.short	0x1a
	.short	0x1603
	.short	0x5
	.long	0x1065
	.long	0x1067
	.long	0x1066
	.long	0x1067
	.long	0x1067
	.byte	242
	.byte	241
