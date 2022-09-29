#![no_std]
#![no_main]

extern crate kzios_kernel;

#[export_name = "dtb_addr"]
pub static DTB: &[u8] = include_bytes!("../device.dtb");

#[export_name = "init0_addr"]
pub static INIT0: &[u8] = include_bytes!("../../../artifacts/kzios_init0");

#[export_name = "init0_size"]
pub static INIT0_SIZE: usize = INIT0.len();
