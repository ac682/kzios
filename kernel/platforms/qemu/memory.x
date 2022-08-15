MEMORY
{
    RAM (rwx) : ORIGIN = 0x80000000, LENGTH = 8M
}

STACK_SIZE = 0xC0000;
TRAP_STACK_SIZE = 0x40000;

REGION_ALIAS("REGION_TEXT", RAM);
REGION_ALIAS("REGION_RODATA", RAM);
REGION_ALIAS("REGION_DATA", RAM);
REGION_ALIAS("REGION_BSS", RAM);
REGION_ALIAS("REGION_STACK", RAM);
REGION_ALIAS("REGION_TRAP_STACK", RAM);