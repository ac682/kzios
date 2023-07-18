#![no_std]
#![feature(lang_items, alloc_error_handler, panic_info_message, is_some_and)]
use core::arch::global_asm;

use external::_hart_num;

use crate::mm::frame;

extern crate alloc;

mod console;
mod external;
mod hart;
mod mm;
mod rt;
mod sbi;
mod sync;
mod trap;

global_asm!(include_str!("assembly.asm"));

const LOGO: &str = include_str!("../logo.txt");

fn main() {
    // only #0 goes here to kernel init(AKA boot)
    println!("{}", LOGO);
    frame::init();
    // device
}
