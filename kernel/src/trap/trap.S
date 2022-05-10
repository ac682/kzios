.option norvc
.altmacro
.set NUM_REGS, 32  # Number of registers per context
.set REG_SIZE, 8   # Register size (in bytes), for I, D(F), all 64 bits

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

.text
.global trap_vector

trap_vector:
	# 保存寄存器
	csrrw	t6,sscratch,t6 # 交换 t6 和 sscartch， t6 指向陷入帧
	.set	i,0
	.rept	NUM_REGS-1 # 保存前 31 个寄存器，也就是除了 x31
			save_gp	%i,t6
			.set	i,i+1
	.endr

	mv		t5,t6 # 现在 t5 指向陷入帧
	csrr	t6,sscratch # 复原 t6
	save_gp 31,t5

	csrw	sscratch,t5 # sscratch 恢复

	.set	i,0
	.rept	NUM_REGS
			save_fp	%i,t5
			.set	i,i+1
	.endr
 
	# 此处应该切换运行时态的 sp，sstatus，satp(页表)，但由于目前还没有用户态，陷入来自于内核态，就直接进吧
	# sp 还是得设置的
	#la t6, ekernel
	#mv sp, t6
	
	# 进入 rust 环境
	call	handle_trap

	# 恢复寄存器
	csrr	t6,sscratch
	.set	i,0
	.rept	NUM_REGS
			load_fp	%i
			.set	i,i+1
	.endr

	.set	i,0
	.rept	NUM_REGS
			load_gp	%i
			.set	i,i+1
	.endr
	#TODO: 可以根据 scause 中的第一位判断是异常还是中断，可以在这里 mepc 后移动
	sret # 跳到 mepc， 如果是异常， rust 中应该把 mepc 往后移