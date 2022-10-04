#![feature(lang_items, alloc_error_handler, panic_info_message, linkage)]
#![no_std]

use core::arch::global_asm;

use board::BoardInfo;
pub use erhino_shared::*;

extern crate alloc;

mod external;
mod mm;
mod rt;
mod trap;
pub mod board;

global_asm!(include_str!("assembly.asm"));


pub fn init(info: BoardInfo){
    println!("boot stage #3: kernel initialization");
    println!("{}", info);
    println!("boot completed");
}