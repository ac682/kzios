#![no_std]

use core::arch::asm;

extern crate rinlib;

fn main() {
    unsafe {
        asm!("ebreak", in("x10") 0);
    }
}
