OUTPUT_ARCH("riscv")
ENTRY(main)

SECTIONS
{
  .text : ALIGN(16) {
    *(.text)
  }

  .rodata : ALIGN(16) {
    *(.srodata)
    *(.rodata)
  }

  .data : ALIGN(16) {
    *(.sdata)
    *(.data)
  }

  .bss (NOLOAD) : ALIGN(16) {
    PROVIDE(_bss_start = .);
    *(.sbss)
    *(.bss)
    PROVIDE(_bss_end = .);
  }

  /DISCARD/ : {
    *(.eh_frame)
  }
}