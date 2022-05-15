.DEFAULT_GOAL := run

KERNEL_NAME = kzios-kernel

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

# QEMU
QEMU_MEMORY ?= 512
QEMU_CORES ?= 1
QEMU_DEBUG ?= gdb

# K210
K210_BOOTLOADER_SIZE = 131072
K210_SERIALPORT	= /dev/ttyUSB1
K210_BURNER	= kflash

# KERNEL ENTRY
ifeq ($(BOARD), qemu)
	KERNEL_ENTRY_PA := 0x80200000
else ifeq ($(BOARD), k210)
	KERNEL_ENTRY_PA := 0x80020000
endif

# Binutils
OBJDUMP := rust-objdump --arch-name=riscv64
OBJCOPY := rust-objcopy --binary-architecture=riscv64

clean:
	@cargo clean

build:
	@cp src/linker-$(BOARD).ld src/linker.ld
	@cargo build --release
	@$(OBJCOPY) --strip-all $(KERNEL_ELF) -O binary $(KERNEL_BIN)
	@rm src/linker.ld
run: build
ifeq ($(BOARD),qemu)
	@qemu-system-riscv64 \
    	-machine virt \
    	-smp cores=$(QEMU_CORES) \
    	-m $(QEMU_MEMORY) \
    	-nographic \
    	-bios $(BOOTLOADER) \
    	-drive id=sdcard0,if=none,format=raw,file=./sdcard.img \
    	-device sdhci-pci \
    	-device sd-card,drive=sdcard0 \
    	-device loader,file=$(KERNEL_ELF).bin,addr=$(KERNEL_ENTRY_PA)
else
	@cp $(BOOTLOADER) $(BOOTLOADER)-merged.bin
	@dd if=$(KERNEL_BIN) of=$(BOOTLOADER)-merged.bin bs=$(K210_BOOTLOADER_SIZE) seek=1
	@$(K210_BURNER) -p $(K210_SERIALPORT) -B goE -b 1500000 $(BOOTLOADER)-merged.bin
	@python3 -m serial.tools.miniterm --eol LF --dtr 0 --rts 0 --filter direct $(K210_SERIALPORT) 115200
endif

debug: build
	@tmux new-session -d \
		"qemu-system-riscv64 -machine virt -nographic -bios $(BOOTLOADER) -device loader,file=$(KERNEL_BIN),addr=$(KERNEL_ENTRY_PA) -s -S" && \
		tmux split-window -h "riscv64-elf-gdb -ex 'file $(KERNEL_ELF)' -ex 'set arch riscv:rv64' -ex 'target remote localhost:1234'" && \
		tmux -2 attach-session -d