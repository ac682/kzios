#![no_std]
#![no_main]

extern crate kzios_kernel;


#[export_name = "dtb_addr"]
pub static DTB: &[u8] = include_bytes!("../device.dtb");