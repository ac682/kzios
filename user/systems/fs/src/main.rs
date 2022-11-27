#![no_std]

use rinlib::prelude::*;

mod fs;
mod impls;
mod tree;

extern crate alloc;
extern crate rinlib;

fn main() {
    dbg!("File System Ready\n");
}
