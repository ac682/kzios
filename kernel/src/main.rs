#![no_std]
#![no_main]

mod lang_items;
mod uart;

use core::arch::global_asm;
global_asm!(include_str!("boot.S"));

#[no_mangle]
extern "C" fn main() -> !
{
    loop{}
}