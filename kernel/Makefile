.DEFAULT_GOAL := Run

KERNEL_NAME = kzios-kernel

# Building
TARGET := riscv64gc-unknown-none-elf
MODE := release
KERNEL_ELF := target/$(TARGET)/$(MODE)/$(KERNEL_NAME)
KERNEL_BIN := $(KERNEL_ELF).bin
SDCARD := ./sdcard.img

# Running

CORES ?= 1

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
    	-smp cores=${CORES} \
    	-m 32 \
    	-nographic \
    	-bios $(BOOTLOADER) \
    	-drive id=sdcard0,if=none,format=raw,file=./sdcard.img \
    	-device sdhci-pci \
    	-device sd-card,drive=sdcard0 \
    	-device loader,file=$(KERNEL_ELF).bin,addr=${KERNEL_ENTRY_PA}
