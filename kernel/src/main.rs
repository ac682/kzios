#![no_std]
#![no_main]
#![feature(panic_info_message)]
#![feature(alloc_error_handler)]

extern crate alloc;

use core::arch::global_asm;
use sbi::shutdown;

#[macro_use]
mod console;
mod lang_items;
mod sbi;
mod mm;
mod config;
mod device_tree;

global_asm!(include_str!("entry.asm"));

#[no_mangle]
fn entry(hartid: usize, device_tree_pddr: usize) -> ! {
    clear_bss();
    println!("hartid: {}, device_tree_addr: 0x{:x}", hartid.clone(), device_tree_pddr);
    mm::init();
    device_tree::init(device_tree_pddr);
    println!("\x1b[31mkzios\x1b[0m");
    shutdown();
}

fn clear_bss() {
    extern "C" {
        fn sbss();
        fn ebss();
    }
    (sbss as usize..ebss as usize).for_each(|a| unsafe { (a as *mut u8).write_volatile(0) })
}
