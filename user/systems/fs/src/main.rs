#![no_std]

use core::arch::asm;

use alloc::vec::Vec;
use rinlib::prelude::*;

use fs::FileSystem;

use impls::memory::MemoryFs;

mod fs;
mod impls;

extern crate alloc;
extern crate rinlib;

fn main() {
    let mut fs = MemoryFs::new("/");
    fs.make_directory("hello").unwrap();
    fs.print();
}
