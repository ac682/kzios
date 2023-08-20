#![no_std]
#![feature(lang_items, alloc_error_handler, panic_info_message)]
#![allow(internal_features)]

use core::arch::global_asm;

pub use erhino_shared as shared;

extern crate alloc;

pub mod console;
mod external;
mod hart;
mod mm;
mod rt;
pub mod sbi;
mod sync;
mod task;
mod timer;
mod trap;

global_asm!(include_str!("assembly.asm"));