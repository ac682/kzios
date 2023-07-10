#![no_std]
#![feature(lang_items, alloc_error_handler)]

use core::arch::global_asm;

mod rt;

global_asm!(include_str!("assembly.asm"));

fn main() {
    //println!("Hello");
}
