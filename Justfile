# rust
MODE := "debug"
RELEASE := if MODE == "release" { "--release" } else { "" }

# platform
PLATFORM := "qemu"
SBI := "rustsbi"
BOOTLOADER := invocation_directory()/"os/platforms"/PLATFORM/SBI+"-"+PLATFORM
LINKER_SCRIPT := invocation_directory()/"os/platforms"/PLATFORM/"memory.ld"

# compile
RUSTFLAGS_OS := "-Clink-arg=-Tplatforms/linker.ld -Clinker=riscv64-elf-ld"
RUSTFLAGS_USR := ""

TARGET_OS := "riscv64gc-unknown-none-elf"
TARGET_USER := "riscv64gc-unknown-erhino-elf"
TARGET_DIR := invocation_directory()/"artifacts"

OS_ELF := TARGET_DIR/"erhino_kernel"
OS_BIN := OS_ELF+".bin"

# qemu
QEMU_CORES := "4"
QEMU_MEMORY := "128m"
QEMU_LAUNCH := "qemu-system-riscv64 -smp cores="+QEMU_CORES+" -M "+QEMU_MEMORY+" -machine virt -nographic -bios \""+BOOTLOADER+"\" -device loader,file=\""+OS_ELF+"\",addr=0x80200000"

alias b := build
alias c := clean
alias r := run

clean:
    #!/usr/bin/env bash
    if [ -d "artifacts" ]; then
    	rm -r artifacts
    fi

artifact_dir:
    #!/usr/bin/env bash
    if [ ! -d "artifacts" ]; then
    	mkdir artifacts
    fi

build_kernel: artifact_dir
    @echo -e "\033[0;36mBuild: {{PLATFORM}}\033[0m"
    @cp "{{LINKER_SCRIPT}}" "{{TARGET_DIR}}"
    @cd os && RUSTFLAGS="{{RUSTFLAGS_OS}}" cargo build --bin erhino_kernel {{RELEASE}} -Z unstable-options --out-dir {{TARGET_DIR}}
    @rust-objcopy --strip-all {{OS_ELF}} -O binary {{OS_BIN}} --binary-architecture=riscv64

build: build_kernel
    @echo -e "\033[0;32mBuild Successfully!\033[0m"

run_qemu: build
    @echo -e "\033[0;36mQEMU: Simulating\033[0m"
    @{{QEMU_LAUNCH}}

run:
    @just PLATFORM={{PLATFORM}} MODE={{MODE}} run_{{PLATFORM}}

debug: build
    @tmux new-session -d "{{QEMU_LAUNCH}} -s -S" && tmux split-window -h "riscv64-elf-gdb -ex 'file {{OS_ELF}}' -ex 'set arch riscv:rv64' -ex 'target remote localhost:1234'" && tmux -2 attach-session -d