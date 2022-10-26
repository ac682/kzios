#![no_std]

use core::arch::asm;

extern crate rinlib;

fn main() {
    unsafe{asm!("ebreak");}
}
