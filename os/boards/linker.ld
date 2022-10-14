OUTPUT_ARCH(riscv)
ENTRY(_start)

INCLUDE "../artifacts/memory.ld"

SECTIONS
{
    PROVIDE(_memory_start = ORIGIN(RAM));
    PROVIDE(_stack_size = STACK_SIZE);
    PROVIDE(_hart_num = HART_NUM);

    .text : ALIGN(4) {
        *(.text.init)
        *(.text .text.*)
    } > RAM

    .rodata : ALIGN(4) {
        *(.rodata)
    } > RAM


    .data : ALIGN(4) {
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
        . = . + STACK_SIZE*HART_NUM;
    } > RAM


    . = ALIGN(4k) - 1; # make kernel 4k aligned
    PROVIDE(_kernel_end = .);


    PROVIDE(_memory_end = ORIGIN(RAM) + LENGTH(RAM));
}