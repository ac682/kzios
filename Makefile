BOARD ?= qemu
MODE := debug
TARGET := riscv64gc-unknown-none-elf
KERNEL_ELF = target/$(TARGET)/$(MODE)/board_$(BOARD)
KERNEL_BIN = target/$(TARGET)/$(MODE)/board_$(BOARD).bin
CARGO_BUILD = RUSTFLAGS="-Clink-arg=-T$(CURDIR)/boards/$(BOARD)/linker.ld -Cforce-frame-pointers=yes" cargo build
CARGO_BUILD_INIT = RUSTFLAGS="-Clink-arg=-T$(CURDIR)/kinlib/linker.ld -Clink-arg=--no-gc-sections" cargo build
INIT_ELF = target/$(TARGET)/$(MODE)/kzios_init0

K210_SERIAL_PORT := /dev/ttyUSB1

all: build
	@echo DONE!

build:
ifeq ($(MODE),release)
	@$(CARGO_BUILD) --release
else
	@$(CARGO_BUILD)
endif
	@rust-objcopy --strip-all $(KERNEL_ELF) -O binary $(KERNEL_BIN)

init:
	@cd init0 && $(CARGO_BUILD_INIT) --bin kzios_init0

run: build
ifeq ($(BOARD),qemu)
	@qemu-system-riscv64 \
		-M 6m \
		-machine virt \
		-nographic \
		-bios none \
		-kernel $(KERNEL_BIN)
else
	@kflash -p $(K210_SERIAL_PORT) -B goE -b 115200 $(KERNEL_BIN)
	@python3 -m serial.tools.miniterm --eol LF --dtr 0 --rts 0 --filter direct $(K210_SERIAL_PORT) 115200
endif

debug_remote: build
	@qemu-system-riscv64 \
    		-M 8m \
    		-machine virt \
    		-nographic \
    		-bios none \
    		-kernel $(KERNEL_BIN) \
    		-s -S

debug: build
	@tmux new-session -d \
		"qemu-system-riscv64 -M 6m -machine virt -nographic -bios none -kernel $(KERNEL_BIN) -s -S" && \
		tmux split-window -h "riscv64-elf-gdb -ex 'file $(KERNEL_ELF)' -ex 'set arch riscv:rv64' -ex 'target remote localhost:1234'" && \
		tmux -2 attach-session -d

clean:
	@cargo clean
