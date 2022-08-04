#![no_std]
#![no_main]
#![feature(panic_info_message, alloc_error_handler)]
#![feature(pin_macro)]
#![allow(unused)]

mod lang_items;
mod mm;
mod primitive;
mod trap;
mod process;
mod syscall;

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

use crate::mm::paged::KERNEL_SPACE;
use crate::process::Process;

extern "C" {
    fn _kernel_end();
}

global_asm!(include_str!("assembly.asm"));

#[no_mangle]
extern "C" fn main() -> ! {
    // kernel init
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


fn init0() {
    loop {}
    //syscall(0,0,0);
}

fn syscall(id: usize, arg0: usize, arg1: usize) {
    unsafe { asm!("ecall") };
}
