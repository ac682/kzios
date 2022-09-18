OUTPUT_ARCH(riscv)
ENTRY(_start)

MEMORY
{
    VSPACE (rwx) : ORIGIN = 0x0, LENGTH = 5363336m
}
SECTIONS
{
    .* : {
        .*
    } > VSPACE
}