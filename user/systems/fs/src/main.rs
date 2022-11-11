#![no_std]

use core::arch::asm;

use alloc::{ffi::CString, vec::Vec};
use rinlib::{prelude::*, proc::inspect_myself};

use fs::FileSystem;

mod fs;
mod impls;
mod tree;

extern crate alloc;
extern crate rinlib;

fn main() {
    let process = inspect_myself().unwrap();
    dbg!("{}({}) is me", process.name, process.pid);
}
