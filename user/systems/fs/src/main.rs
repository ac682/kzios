#![no_std]

use core::arch::asm;

use rinlib::call::sys_yield;

extern crate rinlib;

fn main() {
    unsafe {
        for _ in 0..10 {
            asm!("ebreak", in("x10") 1);
            sys_yield();
        }
    }
}