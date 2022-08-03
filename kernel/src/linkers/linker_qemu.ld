OUTPUT_ARCH(riscv)
ENTRY(_start)

STACK_SIZE = 0xC0000;
TRAP_STACK_SIZE = 0x40000;

MEMORY
{
    ram (rwx) : ORIGIN = 0x80000000, LENGTH = 8M
}

SECTIONS
{
    PROVIDE(_memory_start = ORIGIN(ram));
    
    PROVIDE(_kernel_start = ORIGIN(ram));

    .text : {
        *(.text.init)
        *(.text .text.*)
    } > ram

    . = ALIGN(4K);
    .rodata : {
        *(.rodata)
    } > ram

    . = ALIGN(4K);
    .data : {
        *(.data)
    } > ram

    
    . = ALIGN(4K);
    .bss (NOLOAD) : {
        PROVIDE(_bss_start = .);
        *(.bss)
        PROVIDE(_bss_end = .);
    } > ram

    . = ALIGN(4K);
    .stack (NOLOAD) : {
        PROVIDE(_stack_start = .);
        . = . + STACK_SIZE;
        PROVIDE(_stack_end = .);
    }

    . = ALIGN(4k);
    .trap_stack (NOLOAD) : {
        PROVIDE(_trap_stack_start = .);
        . = . + TRAP_STACK_SIZE;
        PROVIDE(_trap_stack_end = .);
    }

    PROVIDE(_kernel_end = .);

    PROVIDE(_memory_end = ORIGIN(ram) + LENGTH(ram));
}