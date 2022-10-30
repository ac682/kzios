.option norvc
.altmacro
.set NUM_REGS, 32
.set REG_SIZE, 8

.macro save_gp i, basereg=t6
	sd	x\i, ((\i)*REG_SIZE)(\basereg)
.endm
.macro load_gp i, basereg=t6
	ld	x\i, ((\i)*REG_SIZE)(\basereg)
.endm
.macro save_fp i, basereg=t6
	fsd	f\i, ((NUM_REGS+(\i))*REG_SIZE)(\basereg)
.endm
.macro load_fp i, basereg=t6
	fld	f\i, ((NUM_REGS+(\i))*REG_SIZE)(\basereg)
.endm
# calculate sp pointer, sp = _kernel_end - _stack_size * mhartid
.macro locate_sp out=sp, tmp=t1 
    csrr    \tmp, mhartid
    la      \out, _stack_size
    mul     \tmp, \tmp, \out
    la      \out, _kernel_end
    sub     \out, \out, \tmp
    #addi    \out, \out, -1
.endm

.section .text.init
.global _start
_start:
    csrr    t0, mhartid
	bnez	t0, 3f
    csrw    satp, zero
0:
    la 		t1, _bss_start
	la		t2, _bss_end
	bgeu	t1, t2, 2f
1:
	sd		zero, (t1)
	addi	t1, t1, 8
	bltu	t1, t2, 1b
2:
    # store args passed from the board bootloader
    la      t1, _env
    sd      a0, 0(t1)
    sd      a1, 8(t1)
    sd      a2, 16(t1)
    sd      a3, 24(t1)
    sd      a4, 32(t1)
    sd      a5, 40(t1)
    sd      a6, 48(t1)
    sd      a7, 56(t1)
3:
    # do hart preinitialization & setup trap context
    # enable interrupt and floating point support
    li		t0, (0b1 << 13) | (0b11 << 11) | (1 << 7) | (1 << 3)
    csrw	mstatus, t0
    li      t0, (1 << 3)
    csrw    mie, t0
    # setup trap
    la      t1, _kernel_trap
    csrw    mscratch, t1
    la      t1, _trap_vector
    csrw    mtvec, t1
    locate_sp
    # park non-zero and #0 jumps into main for kernel initialization 
    csrr    t0, mhartid
	bnez	t0, 4f
    # main function
    la      t1, main
    csrw    mepc, t1
    csrr    a0, mhartid
    mret
4:
	wfi
	j	4b

.section .text
.global _trap_vector
_trap_vector:
    csrrw	t6, mscratch, t6

    .set	i, 0
    .rept	NUM_REGS - 1
            save_gp	%i, t6
            .set	i, i + 1
    .endr

    mv		t5, t6
    csrr	t6, mscratch
    save_gp 31, t5

    # .set	i,0
    # .rept	NUM_REGS
    #         save_fp	%i,t5
    #         .set	i,i+1
    # .endr

    csrr    t6, satp
    sd      t6, 512(t5)
    csrr    t6, mstatus
    sd      t6, 520(t5)
    csrr    t6, mepc
    sd      t6, 528(t5)

    csrw	mscratch, t5

    # 进入 rust 环境
    csrr    a0, mhartid
    csrr	a1, mcause
    locate_sp
    call    handle_trap
    csrw    mscratch, a0

.section .text
.global _switch_to_user
.global _enter_user_breakpoint
_switch_to_user:
    # 恢复寄存器
    csrr	t6, mscratch

    # 复原 satp 和 mstatus
    ld      t5, 512(t6)
    csrw    satp, t5
    ld      t5, 520(t6)
    csrw    mstatus, t5
    ld      t5, 528(t6)
    csrw    mepc, t5

    # .set	i,0
    # .rept	NUM_REGS
    # 		load_fp	%i
    # 		.set	i,i+1
    # .endr

    .set	i , 0
    .rept	NUM_REGS
        load_gp	%i
        .set	i, i + 1
    .endr
    sfence.vma
_enter_user_breakpoint:
    mret