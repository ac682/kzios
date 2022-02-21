.DEFAULT_GOAL := Run

KERNEL_NAME = kzios-boot

# Building
TARGET := riscv64gc-unknown-none-elf
MODE := release
KERNEL_ELF := target/$(TARGET)/$(MODE)/$(KERNEL_NAME)
KERNEL_BIN := $(KERNEL_ELF).bin
SDCARD := ./sdcard.img

# Board
BOARD ?= qemu
SBI ?= rustsbi
BOOTLOADER = ./bootloader/$(SBI)-$(BOARD).bin

# KERNEL ENTRY
ifeq ($(BOARD), qemu)
	KERNEL_ENTRY_PA := 0x80200000
else ifeq ($(BOARD), k210)
	KERNEL_ENTRY_PA := 0x80020000
endif

# Binutils
OBJDUMP := llvm-objdump --arch-name=riscv64
OBJCOPY := llvm-objcopy --binary-architecture=riscv64

build:
	@cargo build --release
	@$(OBJCOPY) --strip-all $(KERNEL_ELF) -O binary $(KERNEL_BIN)

run: build
	@qemu-system-riscv64 \
    	-machine virt \
    	-smp cores=1 \
    	-m 32 \
    	-nographic \
    	-bios ./bootloader/rustsbi-qemu.bin \
    	-drive id=sdcard0,if=none,format=raw,file=./sdcard.img \
    	-device sdhci-pci \
    	-device sd-card,drive=sdcard0 \
    	-device loader,file=./target/riscv64gc-unknown-none-elf/release/kzios-boot.bin,addr=0x80200000
