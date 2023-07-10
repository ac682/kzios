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
.endm

.section .text.init
.global _start
_start:
# 0-1 clear bss
0:
    la 		t1, _bss_start
	la		t2, _bss_end
	bgeu	t1, t2, 2f
1:
	sd		zero, (t1)
	addi	t1, t1, 8
	bltu	t1, t2, 1b
# 2 hart interrupt setup
2:
# 3 park non-zero harts
3:
    # a0: hartid, a1: DTB physical address
	bnez	a0, park
4:    
    la sp,_kernel_end
    call main
park:
    wfi
    j park