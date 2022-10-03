MODE := "debug"
RELEASE := if MODE == "release" { "--release" } else { "" }
BOARD := "qemu"
RUSTFLAGS_OS := "'-Clink-arg=-T"+invocation_directory()+"/os/boards"/BOARD/"linker.ld -Cforce-frame-pointers=yes'"
RUSTFLAGS_USER := ""

artifact_dir:
    #!/usr/bin/env bash
    if [ ! -d "artifacts" ]; then
    	mkdir artifacts
    fi

build_user: artifact_dir
    @cd user && RUSTFLAGS={{RUSTFLAGS_USER}} cargo build --bin kzios_init0 {{RELEASE}}
