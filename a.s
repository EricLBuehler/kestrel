	.text
	.file	"program.ke"
	.globl	main                            # -- Begin function main
	.p2align	4, 0x90
	.type	main,@function
main:                                   # @main
.Lfunc_begin0:
	.cfi_sections .debug_frame
	.cfi_startproc
# %bb.0:
	pushq	%rbp
	.cfi_def_cfa_offset 16
	.cfi_offset %rbp, -16
	movq	%rsp, %rbp
	.cfi_def_cfa_register %rbp
                                        # kill: killed $rsi
                                        # kill: killed $edi
	movl	$1, %eax
	addl	$2, %eax
	seto	%cl
	testb	$1, %cl
	jne	.LBB0_1
	jmp	.LBB0_2
.LBB0_1:
	movq	%rsp, %rax
	movq	%rax, %rdi
	addq	$-64, %rdi
	movq	%rdi, %rsp
	movb	$0, -14(%rax)
	movb	$10, -15(%rax)
	movb	$48, -16(%rax)
	movb	$49, -17(%rax)
	movb	$58, -18(%rax)
	movb	$49, -19(%rax)
	movb	$58, -20(%rax)
	movb	$101, -21(%rax)
	movb	$107, -22(%rax)
	movb	$46, -23(%rax)
	movb	$109, -24(%rax)
	movb	$97, -25(%rax)
	movb	$114, -26(%rax)
	movb	$103, -27(%rax)
	movb	$111, -28(%rax)
	movb	$114, -29(%rax)
	movb	$112, -30(%rax)
	movb	$32, -31(%rax)
	movb	$32, -32(%rax)
	movb	$32, -33(%rax)
	movb	$32, -34(%rax)
	movb	$10, -35(%rax)
	movb	$33, -36(%rax)
	movb	$119, -37(%rax)
	movb	$111, -38(%rax)
	movb	$108, -39(%rax)
	movb	$102, -40(%rax)
	movb	$114, -41(%rax)
	movb	$101, -42(%rax)
	movb	$118, -43(%rax)
	movb	$111, -44(%rax)
	movb	$32, -45(%rax)
	movb	$110, -46(%rax)
	movb	$111, -47(%rax)
	movb	$105, -48(%rax)
	movb	$116, -49(%rax)
	movb	$105, -50(%rax)
	movb	$100, -51(%rax)
	movb	$100, -52(%rax)
	movb	$97, -53(%rax)
	movb	$32, -54(%rax)
	movb	$50, -55(%rax)
	movb	$51, -56(%rax)
	movb	$105, -57(%rax)
	movb	$32, -58(%rax)
	movb	$58, -59(%rax)
	movb	$114, -60(%rax)
	movb	$111, -61(%rax)
	movb	$114, -62(%rax)
	movb	$114, -63(%rax)
	movb	$69, -64(%rax)
	callq	printf@PLT
	movl	$4294967295, %eax               # imm = 0xFFFFFFFF
	jmp	.LBB0_3
.LBB0_2:
	jmp	.LBB0_3
.LBB0_3:
	movq	%rsp, %rcx
	movq	%rcx, %rdx
	addq	$-16, %rdx
	movq	%rdx, %rsp
	movl	%eax, -16(%rcx)
	movl	-16(%rcx), %eax
	movq	%rsp, %rcx
	addq	$-16, %rcx
	movq	%rcx, %rsp
	movl	%eax, (%rcx)
	xorl	%eax, %eax
	movq	%rbp, %rsp
	popq	%rbp
	.cfi_def_cfa %rsp, 8
	retq
.Lfunc_end0:
	.size	main, .Lfunc_end0-main
	.cfi_endproc
                                        # -- End function
	.section	".note.GNU-stack","",@progbits
