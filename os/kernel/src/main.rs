#![no_std]
#![feature(lang_items, alloc_error_handler)]

use core::arch::global_asm;

extern crate alloc;

mod rt;
mod hart;
mod external;
mod trap;
mod sbi;
mod console;

global_asm!(include_str!("assembly.asm"));

fn main() {
    // only #0 goes here to kernel init
    println!("Hello, {}", "World");
}
