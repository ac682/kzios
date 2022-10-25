#![feature(
    lang_items,
    alloc_error_handler,
    panic_info_message,
    linkage,
    is_some_and
)]
#![no_std]
#![allow(dead_code)]
#![allow(unused)]

use core::arch::global_asm;

use board::BoardInfo;
pub use erhino_shared::*;

use crate::krn_call::krn_enter_user_space;

extern crate alloc;

// public module should be initialized and completely available before board main function
pub mod board;
pub mod console;
pub mod env;
mod external;
mod krn_call;
mod mm;
mod peripheral;
mod pmp;
pub mod proc;
mod rt;
pub mod sync;
mod trap;

global_asm!(include_str!("assembly.asm"));

pub fn kernel_init(info: BoardInfo) {
    println!("boot stage #3: kernel initialization");
    println!("{}", info);
    peripheral::init(&info);
    println!("boot stage #4: prepare user environment");

    // 内核任务完成了， 回收免得 board 占用 uart 设备
    // 把任务转到 console 设备上
}

pub fn kernel_main() {
    println!("boot completed, enter user mode");
    krn_enter_user_space();
}
