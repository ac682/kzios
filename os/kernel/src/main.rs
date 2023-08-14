#![no_std]
#![feature(lang_items, alloc_error_handler, panic_info_message)]

use core::arch::global_asm;

use tar_no_std::TarArchiveRef;

use crate::{hart::add_process, task::proc::Process};

extern crate alloc;

mod console;
mod external;
mod hart;
mod mm;
mod rt;
mod sbi;
mod sync;
mod task;
mod timer;
mod trap;

global_asm!(include_str!("assembly.asm"));

const LOGO: &str = include_str!("../banner.txt");
// 在文件系统未构建时用于测试的文件
const INITFS: &[u8] = include_bytes!("../../../artifacts/initfs.tar");

fn main() {
    // only #0 goes here to kernel init(AKA boot)
    println!("{}", LOGO);
    // device
    // load program with tar-no-std
    let archive = TarArchiveRef::new(INITFS);
    let systems = archive
        .entries();
    for system in systems {
        let process = Process::from_elf(system.data()).unwrap();
        add_process(process);
    }
    println!("\x1b[0;32m=LINK^START=\x1b[0m");
}
