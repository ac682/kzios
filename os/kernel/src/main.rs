#![no_std]
#![feature(lang_items, alloc_error_handler, panic_info_message)]
#![allow(internal_features)]

use core::arch::global_asm;

extern crate alloc;

mod console;
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

pub fn main() {
    println!("\x1b[0;32m=LINK^START=\x1b[0m");
}
