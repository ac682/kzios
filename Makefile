TARGET := riscv64gc-unknown-none-elf
KERNEL_ELF := target/$(TARGET)/release/kzios-kernel
KERNEL_BIN := target/$(TARGET)/release/kzios-kernel.bin

all: run
	@echo DONE!


build:
	@cd kernel && cargo build --release
	@rust-objcopy --strip-all $(KERNEL_ELF) -O binary $(KERNEL_BIN)

run: build
	@qemu-system-riscv64 \
		-M 8m \
		-machine virt \
		-bios none \
		-kernel $(KERNEL_BIN)