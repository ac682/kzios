OUTPUT_ARCH(riscv)
ENTRY(_start)

MEMORY
{
    RAM (rwx) : ORIGIN = 0x80000000, LENGTH = 6M
}

HEAP_SIZE = 0xA000;
STACK_SIZE = 0x8000;

SECTIONS
{
    PROVIDE(_memory_start = ORIGIN(RAM));

    .text : {
        *(.text.init)
        *(.text .text.*)
    } > RAM

    . = ALIGN(4);
    .rodata : {
        *(.rodata)
    } > RAM

    . = ALIGN(4);
    .data : {
        *(.data)
    } > RAM


    . = ALIGN(4);
    .bss (NOLOAD) : {
        PROVIDE(_bss_start = .);
        *(.bss)
        PROVIDE(_bss_end = .);
    } > RAM

    . = ALIGN(8);
    .runtime (NOLOAD) : {
        PROVIDE(_heap_start = .);
        . = . + HEAP_SIZE;
        . = ALIGN(8);
        PROVIDE(_separator = .);
        . = . + STACK_SIZE;
        . = ALIGN(8);
        PROVIDE(_stack_end = .);
    } > RAM

    
    . = ALIGN(4k); # make kernel 4k aligned
    PROVIDE(_kernel_end = .);

    PROVIDE(_memory_end = ORIGIN(RAM) + LENGTH(RAM));
}