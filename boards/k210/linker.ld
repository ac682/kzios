OUTPUT_ARCH(riscv)
ENTRY(_start)

MEMORY
{
    SRAM : ORIGIN = 0x80000000, LENGTH = 6M
    AI_SRAM : ORIGIN = 0x80600000, LENGTH = 2M
}

STACK_SIZE = 0xC000;

SECTIONS
{
    PROVIDE(_memory_start = ORIGIN(SRAM));

    PROVIDE(_kernel_start = ORIGIN(SRAM));

    .text : {
        *(.text.init)
        *(.text .text.*)
    } > SRAM

    . = ALIGN(4);
    .rodata : {
        *(.rodata)
    } > SRAM

    . = ALIGN(4);
    .data : {
        *(.data)
    } > SRAM


    . = ALIGN(4);
    .bss (NOLOAD) : {
        PROVIDE(_bss_start = .);
        *(.bss)
        PROVIDE(_bss_end = .);
    } > SRAM

    . = ALIGN(4);
    .stack (NOLOAD) : {
        PROVIDE(_stack_start = .);
        . = . + STACK_SIZE;
        PROVIDE(_stack_end = .);
    } > SRAM

    . = ALIGN(4k); # make kernel 4k aligned

    PROVIDE(_kernel_end = .);

    PROVIDE(_memory_end = ORIGIN(SRAM) + LENGTH(SRAM));
}