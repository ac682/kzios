OUTPUT_ARCH("riscv")
ENTRY(main)

SECTIONS
{
  .text : {
    *(.text .text.*)
  }

  .rodata : {
    *(.rodata .rodata.*)
  }

  .data : {
    *(.data .data.*)
  }

  .bss : {
    PROVIDE(_bss_start = .);
    *(.bss .bss.*)
    PROVIDE(_bss_end = .);
  }

  . = ALIGN(4k);
  PROVIDE(_segment_break = .);
}