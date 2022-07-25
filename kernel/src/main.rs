#![no_std]
#![no_main]
#![feature(panic_info_message, alloc_error_handler)]

mod lang_items;
mod primitive;
mod mm;

#[macro_use]
extern crate lazy_static;

extern crate alloc;

use core::{arch::global_asm};

use mm::heap_allocator;
use primitive::qemu;

global_asm!(include_str!("boot.S"));

#[no_mangle]
extern "C" fn main() -> ! {

    qemu::init();
    heap_allocator::init();

    println!("Hello, World!");

    loop {
    }
}
