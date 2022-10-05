OUTPUT_ARCH(riscv)
ENTRY(_start)

MEMORY
{
    RAM (rwx) : ORIGIN = 0x80000000, LENGTH = 6M
}

HEAP_SIZE = 0xC000;
HART_NUM = 2;
STACK_SIZE = 0x4000*HART_NUM;

SECTIONS
{
    PROVIDE(_memory_start = ORIGIN(RAM));

    .text : ALIGN(4) {
        *(.text.init)
        *(.text .text.*)
    } > RAM

    .rodata : ALIGN(4) {
        *(.rodata)
    } > RAM

    . = ALIGN(4);
    .data : {
        *(.data)
    } > RAM


    .bss (NOLOAD) : ALIGN(4) {
        PROVIDE(_bss_start = .);
        *(.bss)
        PROVIDE(_bss_end = .);
    } > RAM

    .runtime (NOLOAD) : ALIGN(4) {
        . = ALIGN(8);
        PROVIDE(_heap_start = .);
        . = . + HEAP_SIZE;
        . = ALIGN(8);
        PROVIDE(_stack_start = .);
        . = . + STACK_SIZE;
        . = ALIGN(4k); # make kernel 4k aligned
        PROVIDE(_kernel_end = .);
    } > RAM
    
    PROVIDE(_memory_end = ORIGIN(RAM) + LENGTH(RAM));
}