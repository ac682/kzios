#![no_std]

use core::fmt::Arguments;

use alloc::string::ToString;
use erhino_kernel::prelude::*;

extern crate alloc;

fn main() {
    // 1008mhz
    let base_frequency = 1_008_000_000usize;
    let clint_base = 0x4000000;
    let info = BoardInfo{
        name: "Mango MQ-R".to_string(),
        base_frequency,
        mswi_address: clint_base,
        mtimer_address: clint_base + 0x4000
    };
    kernel_init(info);

    kernel_main();
}

#[export_name = "board_write"]
pub fn board_write(args: Arguments){}

#[export_name = "board_init"]
pub fn board_init(){}