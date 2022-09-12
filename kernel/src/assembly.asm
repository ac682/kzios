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
    csrr	t0, mhartid
	bnez	t0, 3f
    csrw    satp, zero
    la      sp, _stack_end
    la 		t1, _bss_start
	la		t2, _bss_end
	bgeu	t1, t2, 2f
1:
	sd		zero, (a0)
	addi	t1, t1, 8
	bltu	t1, t2, 1b
2:
    li		t0, (0b11 << 11) | (1 << 7) | (1 << 3)
    csrw	mstatus, t0
    la      t1, main
    csrw    mepc, t1
    csrr    a0, mhartid
    ld      a1, dtb_addr
    mret
3:
	wfi
	j	3b

.section .text
.global _m_trap_vector
_m_trap_vector:
    # 保存通用寄存器
    csrrw	t6, mscratch, t6 # 交换 t6 和 mscratch， t6 指向陷入帧, mscratch 是 t6 原内容

    .set	i,0
    .rept	NUM_REGS-1 # 保存前 31 个寄存器，也就是除了 x31
            save_gp	%i,t6
            .set	i,i+1
    .endr

    mv		t5, t6 # 现在 t5 指向陷入帧
    csrr	t6, mscratch # 复原 t6
    save_gp 31, t5 # 保存 t6

    # .set	i,0
    # .rept	NUM_REGS
    #         save_fp	%i,t5
    #         .set	i,i+1
    # .endr

    # 保存 satp 和 mstatus
    csrr    t6, satp
    sd      t6, 512(t5)
    csrr    t6, mstatus
    sd      t6, 520(t5)

    csrw	mscratch, t5 # mscratch 恢复

    # 进入 rust 环境
    csrr	a0, mscratch
    csrr    a1, mepc
    la      sp, _stack_end
    call    handle_machine_trap
    # csrw    mepc, a0 # set by rust code

.section .text
.global _switch_to_user
_switch_to_user:
    # 恢复寄存器
    csrr	t6, mscratch

    # 复原 satp 和 mstatus
    ld      t5, 512(t6)
    csrw    satp, t5
    ld      t5, 520(t6)
    csrw    mstatus, t5

    # .set	i,0
    # .rept	NUM_REGS
    # 		load_fp	%i
    # 		.set	i,i+1
    # .endr

    # 复原包括 t6 在内的通用寄存器
    .set	i,0
    .rept	NUM_REGS
        load_gp	%i
        .set	i,i+1
    .endr
    sfence.vma
    mret