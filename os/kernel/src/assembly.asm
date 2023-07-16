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

.section .text.init
.global _start
_start:
# 0 hart stack pointer & interrupt setup
0:
    # locate stack pointer
    mv      t0, a0
    la      sp, _stack_size
    mul     t0, t0, sp
    la      sp, _kernel_end
    sub     sp, sp, t0
    # jump into rust_start
    call    main

.section .text
.global _park
_park:
    wfi
    j       _park

.section .text
.global _trap_vector
_trap_vector:
     csrrw	t6, sscratch, t6

    .set	i, 0
    .rept	NUM_REGS - 1
            save_gp	%i, t6
            .set	i, i + 1
    .endr

    mv		t5, t6
    csrr	t6, sscratch
    save_gp 31, t5

    # save floating registers
    csrr    t0, sstatus
    srliw   t0, t0, 13
    andi    t0, t0, 3
    li      t1, 3
    bne     t0, t1, 6f

    .set	i,0
    .rept	NUM_REGS
            save_fp	%i,t5
            .set	i,i+1
    .endr

    # make floating dirty bit
    csrr    t0, sstatus
    li      t1, 1
    slliw   t1, t1, 13
    not     t1, t1
    and     t0, t0, t1
    csrw    mstatus, t0
6:
    csrr    t6, satp
    sd      t6, 512(t5)
    csrr    t6, sepc
    sd      t6, 520(t5)

    csrw	sscratch, t5

    # 进入 rust 环境
    csrr    a0, sscratch
    csrr	a1, scause
    csrr    a2, stval
    # locate sp
    ld      t0, 528(a0)
    la      sp, _stack_size
    mul     t0, t0, sp
    la      sp, _kernel_end
    sub     sp, sp, t0
    # get in
    call    handle_trap
    csrw    sscratch, a0