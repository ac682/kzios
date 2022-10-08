#![feature(lang_items, alloc_error_handler, panic_info_message, linkage)]
#![no_std]
#![allow(dead_code)]

use core::{
    arch::{self, global_asm},
    slice::from_raw_parts,
};

use alloc::vec::Vec;
use board::BoardInfo;
pub use erhino_shared::*;
use tar_no_std::TarArchiveRef;

extern crate alloc;
#[macro_use]
extern crate lazy_static;

// public module should be initialized and completely available before board main function
pub mod board;
pub mod console;
pub mod env;
mod external;
mod mm;
mod peripheral;
mod pmp;
mod process;
mod rt;
mod schedule;
pub mod sync;
mod trap;

global_asm!(include_str!("assembly.asm"));

// 测试用，日后 initfs 应该由 board crate 提供
// board crate 会在 artifacts 里选择部分包括驱动添加到 initfs 里
const INITFS: &[u8] = include_bytes!("../../../artifacts/initfs.tar");

pub fn init(info: BoardInfo) {
    println!("boot stage #3: kernel initialization");
    println!("{}", info);
    peripheral::init(info);
    println!("boot stage #4: prepare user environment");
    println!("boot completed, enter user mode");

    // extract initfs
    let archive = TarArchiveRef::new(INITFS);
    // run drivers
    let drivers = archive
        .entries()
        .filter(|it| it.filename().starts_with("driver_"));
    for driver in drivers{
        println!("driver {}@{}kb", driver.filename(), driver.size() / 1024);
    }
    if let Some(user_init) = archive
        .entries()
        .find(|it| it.filename().as_str() == "user_init")
    {
        println!("init {}@{}kb", user_init.filename(), user_init.size() / 1024);
    }

    // 内核任务完成了， 回收免得 board 占用 uart 设备
    // 把任务转到 console 设备上
}
