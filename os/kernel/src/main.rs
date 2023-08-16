#![no_std]
#![feature(lang_items, alloc_error_handler, panic_info_message)]
#![allow(internal_features)]

use core::arch::global_asm;

extern crate alloc;

mod console;
mod driver;
mod external;
mod hart;
mod mm;
mod rt;
mod sbi;
mod sync;
mod task;
mod timer;
mod trap;

global_asm!(include_str!("assembly.asm"));

const LOGO: &str = include_str!("../banner.txt");

fn main() {
    println!("{}", LOGO);
    println!("\x1b[0;32m=LINK^START=\x1b[0m");
    println!("\x1b[0;33m=SEE^YOU^NEXT^TIME=\x1b[0m");
    sbi::system_reset(0, 0).expect("system reset failure");
}
