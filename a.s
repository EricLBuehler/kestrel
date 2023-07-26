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
	xorl	%eax, %eax
	testb	%al, %al
	jne	.LBB0_2
# %bb.1:
	pushq	%rax
	.cfi_def_cfa_offset 16
	movl	$.Lstr, %edi
	callq	puts@PLT
	addq	$8, %rsp
	.cfi_def_cfa_offset 8
.LBB0_2:
	xorl	%eax, %eax
	retq
.Lfunc_end0:
	.size	main, .Lfunc_end0-main
	.cfi_endproc
                                        # -- End function
	.type	.Lstr,@object                   # @str
	.section	.rodata.str1.1,"aMS",@progbits,1
.Lstr:
	.asciz	"Error: i32 addition overflow!\n    program.ke:1:12"
	.size	.Lstr, 50

	.section	".note.GNU-stack","",@progbits
