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
    mv t0, a0
    la sp, _stack_size
    mul t0, t0, sp
    la sp, _kernel_end
    sub sp, sp, t0
    # jump into rust_start
    call main