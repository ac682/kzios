#![no_std]
#![no_main]
#![feature(panic_info_message, alloc_error_handler)]

mod lang_items;
mod mm;
mod primitive;

#[macro_use]
extern crate lazy_static;

extern crate alloc;

use core::arch::global_asm;

use mm::heaped;
use primitive::qemu;

use crate::mm::paged;

global_asm!(include_str!("boot.S"));

#[no_mangle]
extern "C" fn main() -> ! {
    // kernel init
    heaped::init();
    paged::init();
    // device init
    qemu::init();
    // hello world
    println!("Hello, World!");
    loop {}
}
