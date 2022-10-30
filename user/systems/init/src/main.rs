#![no_std]

use core::arch::asm;

use rinlib::call::sys_yield;

extern crate rinlib;

fn main() {
    unsafe {
        for _ in 0..20 {
            asm!("ebreak", in("x10") 0);
            sys_yield();
        }
    }
}
