MODE := "debug"
RELEASE := if MODE == "release" { "--release" } else { "" }
BOARD := "qemu"
RUSTFLAGS_OS := "-Clink-arg=-T"+invocation_directory()+"/os/boards"/BOARD/"linker.ld -Clinker=riscv64-elf-ld"
# its better to use -Clinker=riscv64-elf-ld as linker in user app which was set to rust-lld in target json
RUSTFLAGS_USER := ""
TARGET_OS := "riscv64gc-unknown-none-elf"
TARGET_USER := "riscv64gc-unknown-erhino-elf"
TARGET_DIR := invocation_directory()/"artifacts"
OS_ELF := TARGET_DIR/"board_"+BOARD
OS_BIN := OS_ELF+".bin"

alias all := run
alias b := build
alias c := clean
alias d := debug
alias r := run

# qemu
QEMU_CORES := "2"
QEMU_MEMORY := "6m"
QEMU_LAUNCH := "qemu-system-riscv64 -smp cores="+QEMU_CORES+" -M "+QEMU_MEMORY+" -machine virt -nographic -bios none -kernel "+OS_BIN

artifact_dir:
    #!/usr/bin/env bash
    if [ ! -d "artifacts" ]; then
    	mkdir artifacts
    fi

build_user: artifact_dir
    @cd user && RUSTFLAGS="{{RUSTFLAGS_USER}}" cargo build --bin system_init {{RELEASE}} -Z unstable-options --out-dir {{TARGET_DIR}}

build_os: artifact_dir
    @cd os && RUSTFLAGS="{{RUSTFLAGS_OS}}" cargo build --bin board_{{BOARD}} {{RELEASE}} -Z unstable-options --out-dir {{TARGET_DIR}}
    @rust-objcopy --strip-all {{OS_ELF}} -O binary {{OS_BIN}} 

build: build_os build_user
    @echo -e "\033[0;32mBuild Successfully!\033[0m"

run_qemu EXPOSE="-s -S": build
    @echo -e "\033[0;36mQEMU: Simulating\033[0m"
    @{{QEMU_LAUNCH}} {{EXPOSE}}

run: (run_qemu "")

debug: build
    @tmux new-session -d "{{QEMU_LAUNCH}} -s -S" && tmux split-window -h "riscv64-elf-gdb -ex 'file {{OS_ELF}}' -ex 'set arch riscv:rv64' -ex 'target remote localhost:1234'" && tmux -2 attach-session -d

clean:
    #!/usr/bin/env sh
    echo Cleaning os workspace...
    cd os && cargo clean
    echo Cleaning user workspace...
    cd ../user && cargo clean
    cd ..
    echo Removing artifacts...
    if [ -d "artfacts" ]; then
        rm -r artifacts
    fi
    echo -e "\033[0;35mDone!\033[0m"
