OUTPUT_ARCH(riscv)
ENTRY(_start)

INCLUDE "../artifacts/memory.ld"

SECTIONS
{
    PROVIDE(_memory_start = ORIGIN(RAM));
    PROVIDE(_stack_size = STACK_SIZE);
    PROVIDE(_hart_num = HART_NUM);

    .text : ALIGN(8) {
        *(.text.init)
        *(.text .text.*)
    } > RAM

    .rodata : ALIGN(8) {
        *(.rodata)
    } > RAM


    .data : ALIGN(8) {
        *(.data)
    } > RAM


    .bss (NOLOAD) : ALIGN(8) {
        PROVIDE(_bss_start = .);
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





    PROVIDE(_memory_end = ORIGIN(RAM) + LENGTH(RAM));
}