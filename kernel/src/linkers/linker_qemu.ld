OUTPUT_ARCH(riscv)
ENTRY(_start)

STACK_SIZE = 0x100000;

MEMORY
{
    ram (rwx) : ORIGIN = 0x80000000, LENGTH = 8M
}

SECTIONS
{
    _kernel_start = .;

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
        _bss_start = .;
        *(.bss)
        _bss_end = .;
    } > ram

    . = ALIGN(4K);
    .stack (NOLOAD) : {
        _stack_start = .;
        . = . + STACK_SIZE;
        _stack_end = .;
    }

    _kernel_end = .;
}