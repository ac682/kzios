#![no_std]
#![feature(lang_items, alloc_error_handler, panic_info_message, is_some_and)]
use core::arch::global_asm;

use external::_hart_num;

use crate::mm::frame;

extern crate alloc;

mod console;
mod external;
mod hart;
mod rt;
mod sbi;
mod trap;
mod mm;
mod sync;

global_asm!(include_str!("assembly.asm"));

const LOGO: &str = include_str!("../logo.txt");

fn main() {
    // only #0 goes here to kernel init(AKA boot)
    println!("{}", LOGO);
    frame::init();
    // device
    awaken_other_harts();
}

fn awaken_other_harts() {
    // by sending ipi to all harts except #0
    hart::send_ipi((1 << _hart_num as usize) - 2);
}
