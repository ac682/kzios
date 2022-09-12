.section .data.dtb
.global dtb_addr
dtb_addr:
    .incbin "boards/qemu/device.dtb"