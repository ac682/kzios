# rust
MODE := "debug"
RELEASE := if MODE == "release" { "--release" } else { "" }

# platform
PLATFORM := "qemu"
MODEL := "sifive_u"
DEBUGGER_OPTIONS := if MODEL == "sifive_u" { "-ex 'add-inferior' -ex 'inferior 2' -ex 'attach 2' -ex 'set schedule-multiple'" } else { "" }
DTS := invocation_directory()/"os/platforms"/PLATFORM/MODEL/"device.dts"
LINKER_SCRIPT := invocation_directory()/"os/platforms/linker.ld"
MEMORY_SCRIPT := invocation_directory()/"os/platforms"/PLATFORM/MODEL/"memory.x"
RENODE_SCRIPT := invocation_directory()/"os/platforms"/PLATFORM/MODEL/"renode.resc"

# compile
RUSTFLAGS_OS := "-Clink-arg=-Tplatforms/linker.ld -Clinker=riscv64-elf-ld"
RUSTFLAGS_USER := ""

TARGET_OS := "riscv64gc-unknown-none-elf"
TARGET_USER := "riscv64gc-unknown-erhino-elf"
TARGET_DIR := invocation_directory()/"artifacts"

KERNEL_ELF := TARGET_DIR/"erhino_kernel"
KERNEL_BIN := KERNEL_ELF+".bin"

DTB := TARGET_DIR/"device.dtb"
SDCARD := TARGET_DIR/"sdcard.img"

OPENSBI_BUILD_DIR := invocation_directory()/"submodules/opensbi/build"

# qemu
QEMU_LAUNCH := "qemu-system-riscv64 -M "+MODEL+" -m 1024M -nographic -kernel '"+KERNEL_ELF+"' -dtb '"+DTB+"' -device loader,file=artifacts/initfs.tar,addr=0xB0000000"

# gdb
GDB_BINARY := "gdb-multiarch"
GDB_TARGET := KERNEL_ELF
# GDB_BINARY := "riscv64-elf-gdb"

alias b := build_kernel
alias c := clean
alias f := flash

clean:
    #!/usr/bin/env bash
    if [ -d "artifacts" ]; then
    	rm -r artifacts
    fi
    cargo clean --manifest-path os/Cargo.toml
    #cargo clean --manifest-path user/Cargo.toml

artifact_dir: 
    #!/usr/bin/env bash
    if [ ! -d "artifacts" ]; then
    	mkdir artifacts
    fi
    if [ ! -d "artifacts/build" ]; then
    	mkdir artifacts/build
    fi
    if [ ! -d "artifacts/initfs" ]; then
    	mkdir artifacts/initfs
    fi

make_dtb: artifact_dir
    @echo Selected DTS {{PLATFORM}}/{{MODEL}}.dts
    @dtc -O dtb -o "{{DTB}}" "{{DTS}}"

make_sdcard: artifact_dir
    #!/usr/bin/env bash
    if [ ! -f "{{SDCARD}}" ]; then
        echo Creating sdcard image
    	qemu-img create '{{SDCARD}}' 128M
    fi

build_user: artifact_dir
    @cd user && RUSTFLAGS="{{RUSTFLAGS_USER}}" cargo build --bins {{RELEASE}} -Z unstable-options --out-dir "{{TARGET_DIR}}/build"
    @echo -e "\033[0;32mUser space programs build successfully!\033[0m"

make_initfs: build_user
    @mkdir -p "{{TARGET_DIR}}/initfs/bin"
    @cp {{TARGET_DIR}}/build/srv_* "{{TARGET_DIR}}/initfs/bin"
    @cp {{TARGET_DIR}}/build/drv_* "{{TARGET_DIR}}/initfs/bin"
    @cd "{{TARGET_DIR}}/initfs" && find . -type f | tar --transform 's/^..//' -cvf ../initfs.tar --files-from=/dev/stdin

build_opensbi options:
    @echo -e "\033[0;36mBuild OpenSBI: {{options}}\033[0m"
    @cd submodules/opensbi && make -j8 CROSS_COMPILE=riscv64-linux-gnu- {{options}}
    @cp {{OPENSBI_BUILD_DIR}}/platform/generic/firmware/fw_*.bin '{{TARGET_DIR}}'
    @cp {{OPENSBI_BUILD_DIR}}/platform/generic/firmware/fw_*.elf '{{TARGET_DIR}}'
    @echo -e "\033[0;32mOpenSBI build successfully!\033[0m"

build_kernel: 
    @echo -e "\033[0;36mBuild kernel: {{PLATFORM}}\033[0m"
    @cp "{{MEMORY_SCRIPT}}" "{{TARGET_DIR}}"
    @cd os && RUSTFLAGS="{{RUSTFLAGS_OS}}" cargo build --bin erhino_kernel {{RELEASE}} -Z unstable-options --out-dir {{TARGET_DIR}}
    @rust-objcopy {{KERNEL_ELF}} -S -O binary {{KERNEL_BIN}} -B=riscv64
    @echo -e "\033[0;32mKernel build successfully!\033[0m"

build_k210: && (build_opensbi "PLATFORM=kendryte/k210 FW_PAYLOAD=y FW_PAYLOAD_OFFSET=0x20000 FW_PAYLOAD_PATH="+KERNEL_BIN+" FW_PAYLOAD_FDT_PATH="+DTB+"") 
    @just PLATFORM=kendryte MODEL=k210 MODE=release build_kernel
    @just PLATFORM=kendryte MODEL=k210 MODE=release make_dtb
    @cp '{{OPENSBI_BUILD_DIR}}/platform/kendryte/k210/firmware/fw_payload.bin' '{{TARGET_DIR}}'

# make_sdcard 可以先稍稍
run_qemu +OPTIONS: make_dtb make_initfs build_kernel
    @echo -e "\033[0;36mQEMU: Simulating\033[0m"
    @{{QEMU_LAUNCH}} {{OPTIONS}}

run_qemu_dump_dtb:
    @{{QEMU_LAUNCH}} -machine dumpdtb="{{TARGET_DIR}}/dump.dtb"
    @dtc -O dts -o "{{TARGET_DIR}}/dump.dts" -I dtb "{{TARGET_DIR}}/dump.dtb"

run_k210: build_k210
    @just PLATFORM=kendryte MODEL=k210 MODE=release run_renode

virt:
    @just PLATFORM=qemu MODEL=virt MODE=debug run_qemu -smp cores=4

sifive_u:
    @just PLATFORM=qemu MODEL=sifive_u MODE=debug run_qemu -smp cores=5

flash_k210: build_k210
    @python3 -m kflash -p /dev/ttyUSB1 -b 1500000 "{{KERNEL_ELF}}_merged.bin"
    @python3 -m serial.tools.miniterm --eol LF --dtr 0 --rts 0 --filter direct /dev/ttyUSB1 115200

flash:
    @just PLATFORM={{PLATFORM}} MODEL={{MODEL}} MODE={{MODE}} flash_{{PLATFORM}}