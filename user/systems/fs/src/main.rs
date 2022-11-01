#![no_std]

use core::arch::asm;

use rinlib::call::{sys_fork, sys_yield};

mod fs;
mod impls;

extern crate rinlib;

fn main() {
    unsafe {
        let pid = sys_fork(0).unwrap();
        for _ in 0..10 {
            asm!("ebreak", in("x10") pid);
            sys_yield();
        }
    }
}
