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
    csrw    satp, zero
    la      sp, _stack_end
    la 		a0, _bss_start
	la		a1, _bss_end
	bgeu	a0, a1, 2f
1:
	sd		zero, (a0)
	addi	a0, a0, 8
	bltu	a0, a1, 1b
2:
    li		t0, (0b11 << 11) | (1 << 7) | (1 << 3)
    csrw	mstatus, t0
    la      t1, main
    csrw    mepc, t1
    mret

.section .text
.global _m_trap_vector
_m_trap_vector:
    # 保存寄存器
    csrrw	t6,mscratch,t6 # 交换 t6 和 mscratch， t6 指向陷入帧, mscratch 是 t6 原内容
    .set	i,0
    .rept	NUM_REGS-1 # 保存前 31 个寄存器，也就是除了 x31
            save_gp	%i,t6
            .set	i,i+1
    .endr

    mv		t5,t6 # 现在 t5 指向陷入帧
    csrr	t6,mscratch # 复原 t6
    save_gp 31,t5 # 保存 t6

    csrw	mscratch,t5 # mscratch 恢复

    # .set	i,0
    # .rept	NUM_REGS
    # 		save_fp	%i,t5
    # 		.set	i,i+1
    # .endr

    # 进入 rust 环境
    # 栈!
    la      t6, _trap_stack_end
    mv      sp, t6

    call    handle_machine_trap

    # 恢复寄存器
    csrr	t6,mscratch

    # .set	i,0
    # .rept	NUM_REGS
    # 		load_fp	%i
    # 		.set	i,i+1
    # .endr

    .set	i,0
    .rept	NUM_REGS
        load_gp	%i
        .set	i,i+1
    .endr

    #TODO: 可以根据 mcause 中的第一位判断是异常还是中断，可以在这里 mepc 后移动
    mret # 跳到 mepc， 如果是异常， rust 中应该把 mepc 往后移

.section .text
.global _switch_to_user
_switch_to_user:
    # a0 - Frame address
    # a1 - Program counter
    ld      t5, 512(a0)
    csrw    satp, t5
    li		t0, 1 << 7 | 1 << 5 # MPP: 0, MPIE: 1, UPIE: 1
    csrw	mstatus, t0
    csrw    mscratch, a0
    csrw    mepc, a1
    la      t1, _m_trap_vector
    csrw    mtvec, t1

    mv	t6, a0 # restore the registers
    .set    i,1
    .rept	31
        load_gp %i,t6
        .set	i,i+1
    .endr

    sfence.vma
    mret