OUTPUT_ARCH("riscv")
ENTRY(main)

SECTIONS
{
  .text : ALIGN(8) {
    *(.text)
  }

  .rodata : ALIGN(8) {
    *(.srodata)
    *(.rodata)
  }

  .data : ALIGN(8) {
    *(.sdata)
    *(.data)
  }

  .bss (NOLOAD) : ALIGN(8) {
    PROVIDE(_bss_start = .);
    *(.sbss)
    *(.bss)
    PROVIDE(_bss_end = .);
  }
}