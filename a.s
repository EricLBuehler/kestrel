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
	pushq	%rbx
	pushq	%rax
	.cfi_offset %rbx, -24
                                        # kill: killed $rsi
                                        # kill: killed $edi
	movl	$1, %ebx
	addl	$2, %ebx
	seto	%al
	testb	$1, %al
	jne	.LBB0_1
	jmp	.LBB0_2
.LBB0_1:
	movabsq	$.L__unnamed_1, %rdi
	callq	printf@PLT
	jmp	.LBB0_3
.LBB0_2:
	jmp	.LBB0_3
.LBB0_3:
	movq	%rsp, %rax
	movq	%rax, %rcx
	addq	$-16, %rcx
	movq	%rcx, %rsp
	movl	%ebx, -16(%rax)
	movl	-16(%rax), %eax
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
	leaq	-8(%rbp), %rsp
	popq	%rbx
	popq	%rbp
	.cfi_def_cfa %rsp, 8
	retq
.Lfunc_end0:
	.size	main, .Lfunc_end0-main
	.cfi_endproc
                                        # -- End function
	.type	.L__unnamed_1,@object           # @0
	.section	.rodata,"a",@progbits
	.p2align	4
.L__unnamed_1:
	.asciz	"Error: i32 addition overflow!\n    program.ke:1:12\n"
	.size	.L__unnamed_1, 51

	.section	".note.GNU-stack","",@progbits
