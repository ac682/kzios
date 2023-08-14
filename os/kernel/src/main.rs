#![no_std]
#![feature(lang_items, alloc_error_handler, panic_info_message)]

use core::{arch::global_asm, ptr::slice_from_raw_parts};

use tar_no_std::TarArchiveRef;

use crate::{
    external::{_ramfs_end, _ramfs_start},
    hart::add_process,
    mm::{frame, page::PAGE_BITS},
    task::proc::Process,
};

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

fn main() {
    // only #0 goes here to kernel init(AKA boot)
    println!("{}", LOGO);
    // device
    // load program with tar-no-std
    let ramfs = unsafe {
        &*slice_from_raw_parts(
            _ramfs_start as usize as *const u8,
            _ramfs_end as usize - _ramfs_start as usize,
        )
    };
    let archive = TarArchiveRef::new(ramfs);
    let systems = archive.entries();
    for system in systems {
        let process = Process::from_elf(system.data()).unwrap();
        add_process(process);
    }
    // create initfs in vfs
    // recycle initfs
    frame::add_frame(
        _ramfs_start as usize >> PAGE_BITS,
        _ramfs_end as usize >> PAGE_BITS,
    );
    println!("\x1b[0;32m=LINK^START=\x1b[0m");
}
