TARGET := riscv64gc-unknown-none-elf
KERNEL_ELF := target/$(TARGET)/release/kzios-kernel
KERNEL_BIN := target/$(TARGET)/release/kzios-kernel.bin

all: build
	@echo DONE!


build:
	@cd kernel && cargo build --release
	@rust-objcopy --strip-all $(KERNEL_ELF) -O binary $(KERNEL_BIN)

run: build
	@qemu-system-riscv64 \
		-M 8m \
		-machine virt \
		-nographic \
		-bios none \
		-kernel $(KERNEL_BIN)

debug: build
	@tmux new-session -d \
		"qemu-system-riscv64 -M 8m -machine virt -nographic -bios none -kernel $(KERNEL_BIN) -s -S" && \
		tmux split-window -h "riscv64-elf-gdb -ex 'file $(KERNEL_ELF)' -ex 'set arch riscv:rv64' -ex 'target remote localhost:1234'" && \
		tmux -2 attach-session -d