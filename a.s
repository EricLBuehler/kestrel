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
	movl	$1, %eax
	addq	$2, %rax
	seto	%cl
	testb	$1, %cl
	jne	.LBB0_1
	jmp	.LBB0_2
.LBB0_1:
	movabsq	$.L__unnamed_1, %rdi
	callq	printf@PLT
                                        # implicit-def: $rax
	jmp	.LBB0_3
.LBB0_2:
	jmp	.LBB0_3
.LBB0_3:
	movq	%rsp, %rcx
	movq	%rcx, %rdx
	addq	$-16, %rdx
	movq	%rdx, %rsp
	movq	%rax, -16(%rcx)
	movq	%rsp, %rax
	movq	%rax, %rcx
	addq	$-16, %rcx
	movq	%rcx, %rsp
	movq	%rdx, -16(%rax)
	movq	%rsp, %rcx
	addq	$-16, %rcx
	movq	%rcx, %rsp
	movq	%rcx, -16(%rax)
	movq	%rsp, %rax
	addq	$-16, %rax
	movq	%rax, %rsp
	movq	%rdx, (%rax)
	movb	$1, %al
	xorl	%ecx, %ecx
	testb	$1, %cl
	jne	.LBB0_4
	jmp	.LBB0_5
.LBB0_4:
	movabsq	$.L__unnamed_2, %rdi
	callq	printf@PLT
                                        # implicit-def: $al
	jmp	.LBB0_6
.LBB0_5:
	jmp	.LBB0_6
.LBB0_6:
	movq	%rsp, %rcx
	movq	%rcx, %rdx
	addq	$-16, %rdx
	movq	%rdx, %rsp
	andb	$1, %al
	movb	%al, -16(%rcx)
	callq	x@PLT
	movq	%rsp, %rbx
	movq	%rbx, %rcx
	addq	$-16, %rcx
	movq	%rcx, %rsp
	andb	$1, %al
	movb	%al, -16(%rbx)
	callq	x@PLT
	movq	%rsp, %rcx
	movq	%rcx, %rdx
	addq	$-16, %rdx
	movq	%rdx, %rsp
	andb	$1, %al
	movb	%al, -16(%rcx)
	movb	-16(%rbx), %al
	movb	-16(%rcx), %cl
	xorb	%cl, %al
	xorb	$1, %al
	movq	%rsp, %rcx
	addq	$-16, %rcx
	movq	%rcx, %rsp
	andb	$1, %al
	movb	%al, (%rcx)
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
	.globl	x                               # -- Begin function x
	.p2align	4, 0x90
	.type	x,@function
x:                                      # @x
.Lfunc_begin1:
	.cfi_startproc
# %bb.0:
	movb	$1, %al
	retq
.Lfunc_end1:
	.size	x, .Lfunc_end1-x
	.cfi_endproc
                                        # -- End function
	.type	.L__unnamed_1,@object           # @0
	.section	.rodata,"a",@progbits
	.p2align	4
.L__unnamed_1:
	.asciz	"Error: std::u64 addition overflow!\n    program.ke:2:17\n"
	.size	.L__unnamed_1, 56

	.type	.L__unnamed_2,@object           # @1
	.p2align	4
.L__unnamed_2:
	.asciz	"Error: std::bool addition overflow!\n    program.ke:6:17\n"
	.size	.L__unnamed_2, 57

	.section	".note.GNU-stack","",@progbits
