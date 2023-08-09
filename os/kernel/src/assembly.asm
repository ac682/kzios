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
    # store hartid in tp resgister (done by SBI but here just to make sure)
    mv      tp, a0 
    # locate stack pointer
    mv      t0, tp
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
.global _kernel_trap
_kernel_trap:
    csrrw	t6, sscratch, t6
    # wont save registers if sscratch == 0
    beqz    t6, 7f

    .set	i, 0
    .rept	NUM_REGS - 1
            save_gp	%i, t6
            .set	i, i + 1
    .endr

    mv		t5, t6
    csrr	t6, sscratch
    save_gp 31, t6

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
    csrw    sstatus, t0
6:
    # save satp and sepc
    csrr    t6, satp
    sd      t6, 512(t5)
    csrr    t6, sepc
    sd      t6, 520(t5)
    # hartid should be restored from 528(TrapFrame) to tp
    ld      tp, 528(t5)

    csrw	sscratch, t5
7:
    # load kernel memory page table
    ld      t0, _kernel_satp
    csrw    satp, t0
    sfence.vma
    # setup sp
    mv      t0, tp
    la      sp, _stack_size
    mul     t0, t0, sp
    la      sp, _kernel_end
    sub     sp, sp, t0
    # go prepare for rust trap handler
    csrr    a0, sscratch
    csrr	a1, scause
    csrr    a2, stval
    # get in
    call    handle_trap
    # save hartid to the next arranged TrapFrame(pointed to by a0) before enter user space to make sure tp in kernel mode is always referring to the hartid
    ld      tp, 528(a0)
    csrw    sscratch, a0
    
.section .text
.global _switch_to_user
.global _enter_user_breakpoint
_switch_to_user:
    # TODO: do registers restore and sret
    mv      t6, a0

    # restore satp and mepc
    ld      t5, 512(t6)
    csrr    t4, satp
    # 如果 t5 t4相等就跳过
    sfence.vma
    csrw    satp, t5
8:
    ld      t5, 520(t6)
    csrw    sepc, t5

    csrr    t0, sstatus
    srliw   t0, t0, 13
    andi    t0, t0, 3
    li      t1, 1
    li      t2, 2
    bne     t0, t1, initial
    bne     t0, t2, clean
initial:
    .set    i, 0
    .rept   NUM_REGS

            .set    i,i+1
    .endr
    j       9f
clean:
    .set	i,0
    .rept	NUM_REGS
    		load_fp	0
    		.set	i,i+1
    .endr
9:
    .set	i , 0
    .rept	NUM_REGS
        load_gp	%i
        .set	i, i + 1
    .endr
_enter_userspace_breakpoin:
    sret

.section .trampoline
.global _user_trap:

    # sscratch holds the trapframe address: (top_number - 1) << 12
    csrrw	t6, sscratch, t6

    .set	i, 0
    .rept	NUM_REGS - 1
            save_gp	%i, t6
            .set	i, i + 1
    .endr

    mv		a0, t6
    csrr	t6, sscratch
    save_gp 31, t6

    # save floating registers
    csrr    t0, sstatus
    srliw   t0, t0, 13
    andi    t0, t0, 3
    li      t1, 3
    bne     t0, t1, 10f

    .set	i,0
    .rept	NUM_REGS
            save_fp	%i,a0
            .set	i,i+1
    .endr

    # clear floating dirty bit
    csrr    t0, sstatus
    li      t1, 1
    slliw   t1, t1, 13
    not     t1, t1
    and     t0, t0, t1
    csrw    sstatus, t0
10:
    # save special register
    csrr    t6, satp
    sd      t6, 512(a0)
    csrr    t6, sepc
    sd      t6, 520(a0)
    # hartid should be restored from 528(TrapFrame) to tp
    ld      tp, 528(a0)
    sret