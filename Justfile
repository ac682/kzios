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
QEMU_LAUNCH := "qemu-system-riscv64 -smp cores="+QEMU_CORES+" -M "+QEMU_MEMORY+" -machine virt -nographic -bios \""+BOOTLOADER+"\" -kernel \""+OS_ELF+"\""

alias b := build_kernel
alias c := clean
alias d := debug
alias r := run
alias run_k210 := run_renode
alias run_mq_r := run_renode

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
    @echo -e "\033[0;32mKernel Build Successfully!\033[0m"

build_k210: build_kernel
    @cp "{{BOOTLOADER}}.bin" "{{OS_BIN}}_merged.bin"
    @dd if="{{OS_BIN}}" of="{{OS_BIN}}_merged.bin" bs=131072 seek=1

run_qemu +EXPOSE="": build_kernel
    @echo -e "\033[0;36mQEMU: Simulating\033[0m"
    @{{QEMU_LAUNCH}} {{EXPOSE}}

run_renode CONSOLE="--console": 
    @just PLATFORM={{PLATFORM}} MODE={{MODE}} build_{{PLATFORM}}
    @echo -e "\033[0;36mRenode console pops up\033[0m"
    @renode {{CONSOLE}} "os/platforms/{{PLATFORM}}/{{PLATFORM}}.resc"

run:
    @just PLATFORM={{PLATFORM}} MODE={{MODE}} run_{{PLATFORM}}

debug: build_kernel
    @tmux new-session -d "{{QEMU_LAUNCH}} -s -S" && tmux split-window -h "riscv64-elf-gdb -ex 'file {{OS_ELF}}' -ex 'set arch riscv:rv64' -ex 'target remote localhost:1234'" && tmux -2 attach-session -d