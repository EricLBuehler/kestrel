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
	subq	$16, %rsp
                                        # kill: killed $rsi
                                        # kill: killed $edi
	leaq	-12(%rbp), %rax
	movq	%rax, -8(%rbp)
# %bb.1:
	movq	%rsp, %rax
	addq	$-16, %rax
	movq	%rax, %rsp
	movl	$2, (%rax)
# %bb.2:
	movq	%rsp, %rax
	addq	$-16, %rax
	movq	%rax, %rsp
	leaq	-8(%rbp), %rcx
	movq	%rcx, (%rax)
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
