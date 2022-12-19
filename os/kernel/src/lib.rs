#![feature(
    lang_items,
    alloc_error_handler,
    panic_info_message,
    linkage,
    is_some_and
)]
#![no_std]
#![allow(dead_code)]

use core::arch::global_asm;

use alloc::string::ToString;
use board::BoardInfo;
use erhino_shared::proc::ProcessPermission;
pub use erhino_shared::*;
use proc::{
    mem::{layout::MemoryLayout, unit::MemoryUnit},
    sch::{smooth::SmoothScheduler, Scheduler},
    Process,
};

use crate::{external::_hart_num, krn_call::krn_enter_user_space, mm::frame};

extern crate alloc;

// public module should be initialized and completely available before board main function
pub mod board;
pub mod console;
pub mod env;
mod external;
mod hart;
mod krn_call;
mod mm;
mod peripheral;
pub mod prelude;
pub mod proc;
mod rt;
pub mod sync;
mod timer;
mod trap;

global_asm!(include_str!("assembly.asm"));

pub fn kernel_init(info: BoardInfo) {
    println!("\x1b[0;34mboot stage #3: kernel initialization\x1b[0m");
    println!("{}", info);
    peripheral::init(&info);
    frame::init();
    hart::init(&info);
    println!("\x1b[0;34mboot stage #4: prepare user environment\x1b[0m");

    // 内核任务完成了， 回收免得 board 占用 uart 设备
    // 把任务转到 console 设备上
}

pub fn kernel_main() {
    println!("\x1b[0;34mboot completed, enter user mode\x1b[0m");
    call_other_harts();
    krn_enter_user_space();
    loop {}
}

fn call_other_harts() {
    if _hart_num as usize > 1 {
        let aclint = peripheral::aclint();
        for i in 1..(_hart_num as usize) {
            aclint.set_msip(i);
        }
    }
}
