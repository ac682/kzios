#![no_std]

#[macro_use]
extern crate kinlib;

use kinlib::{println};

fn main() {
    // TODO: 堆的地方没有被mmap,内核不会去自动分配内存,只能是用户程序手动调用mmap获得内存
    println!("Hello, {}", "App");
    loop {}
}
