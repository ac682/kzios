#![no_std]
#![no_main]
#![feature(panic_info_message, alloc_error_handler)]
#![feature(pin_macro)]

mod lang_items;
mod mm;
mod primitive;
mod trap;

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

extern "C" {
    fn _kernel_end();
}

global_asm!(include_str!("boot.S"));

#[no_mangle]
extern "C" fn main() -> ! {
    // kernel init
    mm::init();
    trap::init();
    // device init
    qemu::init();
    // hello world
    println!("Hello, World!");
    loop {}
}
