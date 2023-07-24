#![no_std]
#![feature(
    lang_items,
    alloc_error_handler,
    panic_info_message,
    is_some_and
)]
#![allow(unused)]

use core::arch::global_asm;

use crate::mm::unit;

extern crate alloc;

mod console;
mod external;
mod hart;
mod mm;
mod rt;
mod sbi;
mod sync;
mod trap;
mod task;

global_asm!(include_str!("assembly.asm"));

const LOGO: &str = include_str!("../logo.txt");

fn main() {
    // only #0 goes here to kernel init(AKA boot)
    println!("{}", LOGO);
    unit::init();
    // device
}
