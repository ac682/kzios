#![feature(lang_items, alloc_error_handler, panic_info_message, linkage)]
#![no_std]
#![allow(dead_code)]

use core::arch::global_asm;

use board::BoardInfo;
pub use erhino_shared::*;

extern crate alloc;
#[macro_use]
extern crate lazy_static;

// public module should be initialized and completely available before board main function
pub mod board;
pub mod console;
pub mod env;
mod external;
mod mm;
mod peripheral;
mod pmp;
mod process;
mod rt;
mod schedule;
pub mod sync;
mod trap;

global_asm!(include_str!("assembly.asm"));

pub fn init(info: BoardInfo) {
    println!("boot stage #3: kernel initialization");
    println!("{}", info);
    peripheral::init(info);
    println!("boot stage #4: prepare user environment");
    println!("boot completed, enter user mode");
}
