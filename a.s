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
	pushq	%r14
	pushq	%rbx
	.cfi_offset %rbx, -32
	.cfi_offset %r14, -24
                                        # kill: killed $rsi
                                        # kill: killed $edi
	movl	$1, %eax
	addl	$2, %eax
	seto	%cl
	testb	$1, %cl
	jne	.LBB0_1
	jmp	.LBB0_2
.LBB0_1:
	movabsq	$.L__unnamed_1, %rdi
	callq	printf@PLT
                                        # implicit-def: $eax
	jmp	.LBB0_3
.LBB0_2:
	jmp	.LBB0_3
.LBB0_3:
	movq	%rsp, %rcx
	movq	%rcx, %r14
	addq	$-16, %r14
	movq	%r14, %rsp
	movl	%eax, -16(%rcx)
	movl	-16(%rcx), %eax
	movq	%rsp, %rdx
	movq	%rdx, %rsi
	addq	$-16, %rsi
	movq	%rsi, %rsp
	movl	%eax, -16(%rdx)
	movl	-16(%rcx), %eax
	movq	%rsp, %rbx
	addq	$-16, %rbx
	movq	%rbx, %rsp
	movl	%eax, (%rbx)
	movl	(%rbx), %eax
	addl	(%rsi), %eax
	seto	%cl
	testb	$1, %cl
	jne	.LBB0_4
	jmp	.LBB0_5
.LBB0_4:
	movabsq	$.L__unnamed_2, %rdi
	callq	printf@PLT
                                        # implicit-def: $eax
	jmp	.LBB0_6
.LBB0_5:
	jmp	.LBB0_6
.LBB0_6:
	movq	%rsp, %rcx
	addq	$-16, %rcx
	movq	%rcx, %rsp
	movl	%eax, (%rcx)
	movl	(%rbx), %eax
	addl	(%r14), %eax
	seto	%al
	testb	$1, %al
	jne	.LBB0_7
	jmp	.LBB0_8
.LBB0_7:
	movabsq	$.L__unnamed_3, %rdi
	callq	printf@PLT
	jmp	.LBB0_9
.LBB0_8:
	jmp	.LBB0_9
.LBB0_9:
	xorl	%eax, %eax
	leaq	-16(%rbp), %rsp
	popq	%rbx
	popq	%r14
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

	.type	.L__unnamed_2,@object           # @1
	.p2align	4
.L__unnamed_2:
	.asciz	"Error: i32 addition overflow!\n    program.ke:4:10\n"
	.size	.L__unnamed_2, 51

	.type	.L__unnamed_3,@object           # @2
	.p2align	4
.L__unnamed_3:
	.asciz	"Error: i32 addition overflow!\n    program.ke:5:1\n"
	.size	.L__unnamed_3, 50

	.section	".note.GNU-stack","",@progbits
