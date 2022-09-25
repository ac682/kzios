.option norvc

.section .text
.global _start
_start:
    la      a0, main
    call    lang_start
    addi    x17, x0, 0x20
    ecall


_signal_return:
    // clean stack
    addi    x17, x0, 0x30
    ecall