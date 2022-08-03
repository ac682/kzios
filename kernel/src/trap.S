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

.section .text
.global _m_trap_vector
_m_trap_vector:
    # 保存寄存器
    csrrw	t6,mscratch,t6 # 交换 t6 和 mscratch， t6 指向陷入帧
    .set	i,0
    .rept	NUM_REGS-1 # 保存前 31 个寄存器，也就是除了 x31
            save_gp	%i,t6
            .set	i,i+1
    .endr

    mv		t5,t6 # 现在 t5 指向陷入帧
    csrr	t6,mscratch # 复原 t6
    save_gp 31,t5

    csrw	mscratch,t5 # mscratch 恢复

    # .set	i,0
    # .rept	NUM_REGS
    # 		save_fp	%i,t5
    # 		.set	i,i+1
    # .endr

    # 进入 rust 环境
    csrr	a0, mepc
    csrr	a1, mtval
    csrr	a2, mcause
    csrr	a3, mhartid
    csrr	a4, mstatus
    mv		a5, t5
    ld		sp, 520(a5)
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