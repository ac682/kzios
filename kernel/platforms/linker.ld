OUTPUT_ARCH(riscv)
ENTRY(_start)

INCLUDE kernel/platforms/qemu/memory.x

SECTIONS
{
    PROVIDE(_memory_start = ORIGIN(RAM));
    
    PROVIDE(_kernel_start = ORIGIN(RAM));

    .text : {
        *(.text.init)
        *(.text .text.*)
    } > REGION_TEXT

    . = ALIGN(4);
    .rodata : {
        *(.rodata)
    } > REGION_RODATA

    . = ALIGN(4);
    .data : {
        *(.data)
    } > REGION_DATA

    
    . = ALIGN(4);
    .bss (NOLOAD) : {
        PROVIDE(_bss_start = .);
        *(.bss)
        PROVIDE(_bss_end = .);
    } > REGION_BSS

    . = ALIGN(4);
    .stack (NOLOAD) : {
        PROVIDE(_stack_start = .);
        . = . + STACK_SIZE;
        PROVIDE(_stack_end = .);
    } > REGION_STACK

    . = ALIGN(4);
    .trap_stack (NOLOAD) : {
        PROVIDE(_trap_stack_start = .);
        . = . + TRAP_STACK_SIZE;
        PROVIDE(_trap_stack_end = .);
    } > REGION_TRAP_STACK

    . = ALIGN(4k); # make kernel 4k aligned

    PROVIDE(_kernel_end = .);

    PROVIDE(_memory_end = ORIGIN(RAM) + LENGTH(RAM));
}