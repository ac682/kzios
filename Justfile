# rust
MODE := "debug"
RELEASE := if MODE == "release" { "--release" } else { "" }

# platform
PLATFORM := "generic"
MODEL := "sifive_u"
DEBUGGER_OPTIONS := if MODEL == "sifive_u" { "-ex 'add-inferior' -ex 'inferior 2' -ex 'attach 2' -ex 'set schedule-multiple'" } else { "" }
FIRMWARE := "jump"
BOOTLOADER := invocation_directory()/"os/opensbi/build/platform"/PLATFORM/"firmware/fw"+"_"+FIRMWARE+".elf"
DTS := invocation_directory()/"os/platforms"/PLATFORM/"models"/MODEL+".dts"
LINKER_SCRIPT := invocation_directory()/"os/platforms"/PLATFORM/"memory.ld"

# compile
RUSTFLAGS_OS := "-Clink-arg=-Tplatforms/linker.ld -Clinker=riscv64-elf-ld"
RUSTFLAGS_USER := ""

TARGET_OS := "riscv64gc-unknown-none-elf"
TARGET_USER := "riscv64gc-unknown-erhino-elf"
TARGET_DIR := invocation_directory()/"artifacts"

KERNEL_ELF := TARGET_DIR/"erhino_kernel"
KERNEL_BIN := KERNEL_ELF+".bin"

DTB := TARGET_DIR/"device.dtb"

# qemu
QEMU_OPTIONS := if MODEL == "sifive_u" { "-smp cores=5 -dtb '"+DTB+"'" } else { "-smp cores=4" }
QEMU_LAUNCH := "qemu-system-riscv64 -M "+MODEL+" -nographic -kernel '"+KERNEL_ELF+"' "+QEMU_OPTIONS

# gdb
GDB_BINARY := "gdb-multiarch"
# GDB_BINARY := "riscv64-elf-gdb"

alias b := build_kernel
alias c := clean
alias d := debug
alias r := run_qemu
alias f := flash

alias run_k210 := run_renode

clean:
    #!/usr/bin/env bash
    if [ -d "artifacts" ]; then
    	rm -r artifacts
    fi

artifact_dir: clean
    #!/usr/bin/env bash
    if [ ! -d "artifacts" ]; then
    	mkdir artifacts
    fi
    if [ ! -d "artifacts/initfs" ]; then
    	mkdir artifacts/initfs
    fi

make_dtb: artifact_dir
    @echo Selected DTS {{PLATFORM}}/{{MODEL}}.dts
    @dtc -O dtb -o "{{DTB}}" "{{DTS}}"

build_user: artifact_dir
    @cd user && RUSTFLAGS="{{RUSTFLAGS_USER}}" cargo build --bins {{RELEASE}} -Z unstable-options --out-dir "{{TARGET_DIR}}/initfs"
    @echo -e "\033[0;32mUser space programs build successfully!\033[0m"

build_initfs: build_user
    @cd "{{TARGET_DIR}}/initfs" && tar -cf ../initfs.tar *

build_opensbi *fw_options="":
    cd os/opensbi && make CROSS_COMPILE=riscv64-linux-gnu- PLATFORM={{PLATFORM}} {{fw_options}}

build_kernel: build_initfs
    @echo -e "\033[0;36mBuild: {{PLATFORM}}\033[0m"
    @cp "{{LINKER_SCRIPT}}" "{{TARGET_DIR}}"
    @cd os && RUSTFLAGS="{{RUSTFLAGS_OS}}" cargo build --bin erhino_kernel {{RELEASE}} -Z unstable-options --out-dir {{TARGET_DIR}}
    @rust-objcopy {{KERNEL_ELF}} -S -O binary {{KERNEL_BIN}} -B=riscv64
    @echo -e "\033[0;32mKernel build successfully!\033[0m"

build_k210: build_kernel 
    @rust-objcopy "{{BOOTLOADER}}" -S -O binary "{{KERNEL_ELF}}_merged.bin"
    @dd if="{{KERNEL_BIN}}" of="{{KERNEL_ELF}}_merged.bin" bs=128k seek=1

run_qemu +EXPOSE="": make_dtb build_kernel
    @echo -e "\033[0;36mQEMU: Simulating\033[0m"
    @{{QEMU_LAUNCH}} {{EXPOSE}}

run_qemu_dump_dtb:
    @{{QEMU_LAUNCH}} -machine dumpdtb="{{TARGET_DIR}}/dump.dtb"
    @dtc -O dts -o "{{TARGET_DIR}}/dump.dts" -I dtb "{{TARGET_DIR}}/dump.dtb"


run_renode CONSOLE="--console": 
    @just PLATFORM={{PLATFORM}} MODE={{MODE}} build_{{PLATFORM}}
    @echo -e "\033[0;36mRenode console pops up\033[0m"
    @renode {{CONSOLE}} "os/platforms/{{PLATFORM}}/{{PLATFORM}}.resc" 

flash_k210: build_k210
    @python3 -m kflash -p /dev/ttyUSB1 -b 1500000 "{{KERNEL_ELF}}_merged.bin"
    @python3 -m serial.tools.miniterm --eol LF --dtr 0 --rts 0 --filter direct /dev/ttyUSB1 115200

flash: 
    @just PLATFORM={{PLATFORM}} MODE={{MODE}} flash_{{PLATFORM}}

debug: make_dtb build_kernel
    @tmux new-session -d "{{QEMU_LAUNCH}} -s -S" && tmux split-window -h "{{GDB_BINARY}} -ex 'set arch riscv:rv64' -ex 'target extended-remote localhost:1234' {{DEBUGGER_OPTIONS}} -ex 'set confirm no' -ex 'file {{KERNEL_ELF}}' -ex 'set confirm yes'" && tmux -2 attach-session -d
