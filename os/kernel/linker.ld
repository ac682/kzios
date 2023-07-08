OUTPUT_ARCH(riscv)
ENTRY(_start)

INCLUDE "../artifacts/memory.ld"

SECTIONS
{
    PROVIDE(_memory_start = ORIGIN(RAM));
    PROVIDE(_kernel_start = ORIGIN(RAM_KERNEL));
    PROVIDE(_stack_size = STACK_SIZE);
    PROVIDE(_hart_num = HART_NUM);

    .text : ALIGN(8) {
        *(.text.init)
        *(.text .text.*)
    } > RAM_KERNEL

    .rodata : ALIGN(8) {
        *(.rodata)
    } > RAM_KERNEL


    .data : ALIGN(8) {
        *(.data)
    } > RAM_KERNEL


    .bss (NOLOAD) : ALIGN(8) {
        PROVIDE(_bss_start = .);
        *(.bss)
        PROVIDE(_bss_end = .);
    } > RAM_KERNEL

    .runtime (NOLOAD) : ALIGN(16) {
        PROVIDE(_heap_start = .);
        . = . + HEAP_SIZE;
        . = ALIGN(16);
        PROVIDE(_stack_start = .);
        . = . + STACK_SIZE * HART_NUM;    
        . = ALIGN(4k); # make kernel 4k aligned
        PROVIDE(_kernel_end = .);
    } > RAM_KERNEL

    PROVIDE(_memory_end = ORIGIN(RAM) + LENGTH(RAM));
}