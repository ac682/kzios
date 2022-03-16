#![no_std]
#![no_main]
#![feature(panic_info_message)]
#![feature(alloc_error_handler)]

extern crate alloc;

use core::arch::{asm, global_asm};
use sbi::shutdown;

#[macro_use]
mod console;
mod config;
mod device_tree;
mod lang_items;
mod mm;
mod sbi;
mod sync;
mod trap;
//mod batch;

global_asm!(include_str!("entry.asm"));
global_asm!(include_str!("link_app.S"));

#[no_mangle]
fn entry(hart_id: usize, device_tree_addr: usize) -> ! {
    clear_bss();
    println!(
        "hart_id: {}, device_tree_addr: {:#x}",
        &hart_id, device_tree_addr
    );
    // memory not initialized, device tree available
    device_tree::init(device_tree_addr);
    mm::init();
    //trap::init();
    //batch::init();
    println!("\x1b[31m[kzios]\x1b[0m");
    shutdown();
}

fn clear_bss() {
    extern "C" {
        fn sbss();
        fn ebss();
    }
    (sbss as usize..ebss as usize).for_each(|a| unsafe { (a as *mut u8).write_volatile(0) })
}
