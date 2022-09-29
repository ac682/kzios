.option norvc

.section .text
.global _start
_start:
    la 		t1, _bss_start
	la		t2, _bss_end
	bgeu	t1, t2, 2f
1:
	sd		zero, (t1)
	addi	t1, t1, 8
	bltu	t1, t2, 1b
2:
    call    main
    # pass a0 (lang_main set) to ecall
    addi    x17, x0, 0x20
    ecall

.section .text
.global _signal_return
_signal_return:
    // clean stack
    addi    x17, x0, 0x30
    ecall