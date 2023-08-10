OUTPUT_ARCH(riscv)
ENTRY(_start)

INCLUDE "../artifacts/memory.ld"

SECTIONS
{
    PROVIDE(_memory_start = ORIGIN(SBI));
    PROVIDE(_kernel_start = ORIGIN(RAM));
    PROVIDE(_stack_size = STACK_SIZE);
    PROVIDE(_hart_num = HART_NUM);

    .text : ALIGN(8) {
        *(.text.init)
        *(.text.*)
        . = ALIGN(0x1000);
        PROVIDE(_trampoline = .);
        *(.trampoline)
        . = ALIGN(0x1000);
        ASSERT(. - _trampoline == 0x1000, "error: _trampoline code larger than one page");
        _text_end = .;
    } > RAM

    .rodata : ALIGN(8) {
        *(.srodata)
        *(.rodata)
    } > RAM


    .data : ALIGN(8) {
        *(.sdata)
        *(.data)
    } > RAM


    .bss (NOLOAD) : ALIGN(8) {
        PROVIDE(_bss_start = .);
        *(.sbss)
        *(.bss)
        PROVIDE(_bss_end = .);
    } > RAM

    .runtime (NOLOAD) : ALIGN(16) {
        PROVIDE(_heap_start = .);
        . = . + HEAP_SIZE;
        . = ALIGN(16);
        PROVIDE(_stack_start = .);
        . = . + STACK_SIZE * HART_NUM;    
        . = ALIGN(4k); # make kernel 4k aligned
        PROVIDE(_kernel_end = .);
    } > RAM

    /DISCARD/ : {
        *(.eh_frame)
    }
    
    PROVIDE(_memory_end = ORIGIN(RAM) + LENGTH(RAM));
}