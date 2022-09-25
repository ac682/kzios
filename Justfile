MODE := "debug"
RELEASE := if MODE == "release" { "--release" } else { "" }
BOARD := "qemu"
TARGET := "riscv64gc-unknown-none-elf"
KERNEL_ELF := "boards/target"/TARGET/MODE/"board_"+BOARD
KERNEL_BIN := "boards/target"/TARGET/MODE/"board_"+BOARD+".bin"
INIT_ELF := "user/target"/TARGET/MODE/"kzios_init0"
RUSTFLAGS_KERNEL := "'-Clink-arg=-T"+invocation_directory()+"/boards"/BOARD/"linker.ld -Cforce-frame-pointers=yes'"
RUSTFLAGS_INIT := "-Clink-arg=-T"+invocation_directory()+"/user/kinlib/linker.ld"

# aliases
alias b := build

# tools
OBJCOPY := "rust-objcopy"
K210_FLASH := "kflash"

# k210

# qemu
QEMU_MEMORY := "6m"
QEMU_CORES := "1"


default:
    @just --list

build_init:
    @cd user && RUSTFLAGS={{RUSTFLAGS_INIT}} cargo build {{RELEASE}}
    @cp {{INIT_ELF}} artifacts

build_os: build_init # 暂时需要, 未来就不要求了
    @cd boards && RUSTFLAGS={{RUSTFLAGS_KERNEL}} cargo build {{RELEASE}}
    @{{OBJCOPY}} --strip-all {{KERNEL_ELF}} -O binary {{KERNEL_BIN}}
    @cp {{KERNEL_ELF}} artifacts
    @cp {{KERNEL_BIN}} artifacts

build: build_init build_os

qemu_debug: build
    @qemu-system-riscv64 \
    -smp cores={{QEMU_CORES}} \
    -M {{QEMU_MEMORY}} \
    -machine virt \
    -nographic \
    -bios none \
    -kernel {{KERNEL_BIN}} \
    -s -S