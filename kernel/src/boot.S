    .section .text.init
    .global _init

_init:
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
3:
    wfi
    j       3b