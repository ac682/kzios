#![no_std]

use core::arch::asm;

use alloc::{ffi::CString, vec::Vec};
use rinlib::prelude::*;

use fs::FileSystem;

mod fs;
mod impls;
mod tree;

extern crate alloc;
extern crate rinlib;

fn main() {
    dbg!("aaa");
}
