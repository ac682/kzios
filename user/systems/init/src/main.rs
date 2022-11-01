#![no_std]

use core::arch::asm;

use rinlib::call::{sys_yield, sys_fork};

extern crate rinlib;

fn main() {
    unsafe {
        for _ in 0..10 {
            asm!("ebreak", in("x10") 0);
            sys_yield();
        }
        let pid = sys_fork(0).unwrap();
        asm!("ebreak", in("x10") pid);
    }
}
