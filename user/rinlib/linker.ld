OUTPUT_ARCH("riscv")
ENTRY(main)

SECTIONS
{
  .text : ALIGN(8) {
    *(.text)
  }

  .rodata : ALIGN(8) {
    *(.rodata)
  }

  .data : ALIGN(8) {
    *(.data)
  }

  .bss (NOLOAD) : ALIGN(8) {
    PROVIDE(_bss_start = .);
    *(.bss)
    PROVIDE(_bss_end = .);
  }

  . = ALIGN(4k);
  PROVIDE(_segment_break = .);
}