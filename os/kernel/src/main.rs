#![no_std]
#![feature(lang_items, alloc_error_handler, panic_info_message)]
use core::arch::global_asm;

use external::_hart_num;

extern crate alloc;

mod console;
mod external;
mod hart;
mod rt;
mod sbi;
mod trap;

global_asm!(include_str!("assembly.asm"));

const LOGO: &str = include_str!("../logo.txt");

fn main() {
    // only #0 goes here to kernel init(AKA boot)
    println!("{}", LOGO);
    awaken_other_harts();
}

fn awaken_other_harts() {
    // by sending ipi
    hart::send_ipi(_hart_num as usize - 2);
}
