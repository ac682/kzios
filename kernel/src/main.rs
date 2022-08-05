#![no_std]
#![no_main]
#![feature(panic_info_message, alloc_error_handler)]
#![feature(pin_macro)]
#![allow(unused)]

mod lang_items;
mod mm;
mod primitive;
mod process;
mod syscall;
mod trap;
mod pmp;
mod external;

#[macro_use]
extern crate lazy_static;

extern crate alloc;

use core::arch::asm;
use core::borrow::Borrow;
use core::{arch::global_asm, panic};

use mm::{
    heaped,
    paged::{self, alloc, page_table::PageTable},
};
use primitive::qemu;
use spin::Mutex;

use crate::process::Process;

global_asm!(include_str!("assembly.asm"));

#[no_mangle]
extern "C" fn main() -> ! {
    // kernel init
    pmp::init();
    mm::init();
    trap::init();
    // device init
    qemu::init();
    // hello world
    println!("Hello, World!");
    let process = Process::new_fn(init0);
    process.activate();
    loop {}
}

#[no_mangle]
fn init0() {
    syscall(0,0,0);
}

fn syscall(id: usize, arg0: usize, arg1: usize) {
    unsafe { asm!("ecall") };
}
