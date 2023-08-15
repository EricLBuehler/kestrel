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
                                        # kill: killed $rsi
                                        # kill: killed $edi
	movl	$123, -20(%rsp)
	leaq	-20(%rsp), %rax
	movq	%rax, -16(%rsp)
	leaq	-16(%rsp), %rax
	movq	%rax, -8(%rsp)
	xorl	%eax, %eax
	retq
.Lfunc_end0:
	.size	main, .Lfunc_end0-main
	.cfi_endproc
                                        # -- End function
	.section	".note.GNU-stack","",@progbits
