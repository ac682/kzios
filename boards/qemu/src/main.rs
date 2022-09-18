#![no_std]
#![no_main]

extern crate kzios_kernel;

#[cfg(debug_assertions)]
#[export_name = "init0"]
pub static INIT0_D: &[u8] =
    include_bytes!("../../../target/riscv64gc-unknown-none-elf/debug/kzios_init0");

#[cfg(not(debug_assertions))]
#[export_name = "init0"]
pub static INIT0_R: &[u8] =
    include_bytes!("../../../target/riscv64gc-unknown-none-elf/release/kzios_init0");

#[export_name = "dtb_addr"]
pub static DTB: &[u8] = include_bytes!("../device.dtb");
