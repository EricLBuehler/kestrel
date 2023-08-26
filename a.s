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
	jmp	.LBB0_4
.LBB0_1:
	movabsq	$.L__unnamed_1, %rdi
	callq	printf@PLT
                                        # implicit-def: $eax
	jmp	.LBB0_5
.LBB0_2:
	jmp	.LBB0_5
.LBB0_3:
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
.LBB0_4:
	.cfi_def_cfa %rbp, 16
	movl	$1, %eax
	incl	%eax
	seto	%cl
	testb	$1, %cl
	jne	.LBB0_1
	jmp	.LBB0_2
.LBB0_5:
	movq	%rsp, %rcx
	addq	$-16, %rcx
	movq	%rcx, %rsp
	movl	%eax, (%rcx)
	jmp	.LBB0_3
.Lfunc_end0:
	.size	main, .Lfunc_end0-main
	.cfi_endproc
                                        # -- End function
	.type	.L__unnamed_1,@object           # @0
	.section	.rodata,"a",@progbits
	.p2align	4
.L__unnamed_1:
	.asciz	"Error: std::i32 addition overflow!\n    program.ke:7:18\n"
	.size	.L__unnamed_1, 56

	.section	".note.GNU-stack","",@progbits
