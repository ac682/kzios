# rust
MODE := "debug"
RELEASE := if MODE == "release" { "--release" } else { "" }

# platform
PLATFORM := "qemu"
MODEL := "virt"
SBI := "rustsbi"
BOOTLOADER := invocation_directory()/"os/platforms"/PLATFORM/SBI+"-"+PLATFORM
DTS := invocation_directory()/"os/platforms"/PLATFORM/"models"/MODEL+".dts"
LINKER_SCRIPT := invocation_directory()/"os/platforms"/PLATFORM/"memory.ld"

# compile
RUSTFLAGS_OS := "-Clink-arg=-Tplatforms/linker.ld -Clinker=riscv64-elf-ld"
RUSTFLAGS_USR := ""

TARGET_OS := "riscv64gc-unknown-none-elf"
TARGET_USER := "riscv64gc-unknown-erhino-elf"
TARGET_DIR := invocation_directory()/"artifacts"

KERNEL_ELF := TARGET_DIR/"erhino_kernel"
KERNEL_BIN := KERNEL_ELF+".bin"

DTB := TARGET_DIR/"device.dtb"

# qemu
QEMU_CORES := "4"
QEMU_MEMORY := "128m"
QEMU_LAUNCH := "qemu-system-riscv64 -smp cores="+QEMU_CORES+" -M "+QEMU_MEMORY+" -machine virt -nographic -bios \""+BOOTLOADER+"\" -kernel \""+KERNEL_ELF+"\" -dtb \""+DTB+"\""

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

make_dtb: artifact_dir
    @echo Selected DTS {{PLATFORM}}/{{MODEL}}.dts
    @dtc -O dtb -o "{{DTB}}" "{{DTS}}"

build_kernel: make_dtb
    @echo -e "\033[0;36mBuild: {{PLATFORM}}\033[0m"
    @cp "{{LINKER_SCRIPT}}" "{{TARGET_DIR}}"
    @cd os && RUSTFLAGS="{{RUSTFLAGS_OS}}" cargo build --bin erhino_kernel {{RELEASE}} -Z unstable-options --out-dir {{TARGET_DIR}}
    @rust-objcopy {{KERNEL_ELF}} -S -O binary {{KERNEL_BIN}} -B=riscv64
    @echo -e "\033[0;32mKernel Build Successfully!\033[0m"

build_k210: build_kernel
    @rust-objcopy "{{BOOTLOADER}}" -S -O binary "{{KERNEL_ELF}}_merged.bin"
    @dd if="{{KERNEL_BIN}}" of="{{KERNEL_ELF}}_merged.bin" bs=128k seek=1

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
    @tmux new-session -d "{{QEMU_LAUNCH}} -s -S" && tmux split-window -h "riscv64-elf-gdb -ex 'file {{KERNEL_ELF}}' -ex 'set arch riscv:rv64' -ex 'target remote localhost:1234'" && tmux -2 attach-session -d