#![no_std]
#![feature(lang_items, alloc_error_handler, panic_info_message)]
#![allow(internal_features)]

use core::{arch::global_asm, ptr::slice_from_raw_parts};

use alloc::format;
use erhino_shared::{
    fal::{DentryAttribute, DentryType},
    path::Path,
};
use tar_no_std::TarArchiveRef;

use crate::{
    external::{_ramfs_end, _ramfs_start},
    hart::SchedulerImpl,
    task::{proc::Process, sched::Scheduler},
};

extern crate alloc;

mod console;
mod external;
mod fal;
mod fs;
mod hart;
mod mm;
mod rng;
mod rt;
mod sbi;
mod sync;
mod task;
mod timer;
mod trap;

const BANNER: &str = include_str!("../banner.txt");

global_asm!(include_str!("assembly.asm"));

pub fn main() {
    println!("{}", BANNER);
    // load program with tar-no-std
    let ramfs = unsafe {
        &*slice_from_raw_parts(
            _ramfs_start as usize as *const u8,
            _ramfs_end as usize - _ramfs_start as usize,
        )
    };
    let archive = TarArchiveRef::new(ramfs);
    let files = archive.entries();
    fs::create(
        Path::from("/initfs").unwrap(),
        DentryType::Directory,
        DentryAttribute::Readable | DentryAttribute::Executable,
    ).unwrap();
    for file in files {
        fs::create_memory_stream(
            Path::from(&format!("/initfs/{}", file.filename())).unwrap(),
            file.data(),
            DentryAttribute::Executable | DentryAttribute::Readable,
        ).unwrap();
        let process = Process::from_elf(file.data()).unwrap();
        SchedulerImpl::add(process, None);
    }
    println!("\x1b[0;32m=LINK^START=\x1b[0m");
}
