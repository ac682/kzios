OUTPUT_ARCH("riscv")
ENTRY(main)

SECTIONS
{
  .text : ALIGN(4) {
    *(.text .text.*)
  }

  .rodata : ALIGN(4) {
    *(.rodata .rodata.*)
  }

  .data : ALIGN(4) {
    *(.data .data.*)
  }

  .bss (NOLOAD) : ALIGN(4) {
    PROVIDE(_bss_start = .);
    *(.bss .bss.*)
    PROVIDE(_bss_end = .);
  }

  . = ALIGN(4k);
  PROVIDE(_segment_break = .);
}