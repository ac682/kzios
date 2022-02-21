#![no_std]
#![no_main]
#![feature(panic_info_message)]

use core::arch::global_asm;

use sbi::shutdown;

#[macro_use]
mod console;
mod lang_items;
mod sbi;
mod interrupt;

global_asm!(include_str!("entry.asm"));

#[no_mangle]
fn entry() -> ! {
    clear_bss();
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
