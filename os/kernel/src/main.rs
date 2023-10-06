#![no_std]
#![feature(lang_items, alloc_error_handler, panic_info_message)]
#![allow(internal_features)]

use core::{arch::global_asm, ptr::slice_from_raw_parts};

use alloc::{format, vec::Vec};
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

static RAMFS: &[u8] = include_bytes!("../../../artifacts/initfs.tar");

pub fn main() {
    println!("{}", BANNER);
    // load program with zip
    let archive = TarArchiveRef::new(RAMFS);
    let files = archive.entries();
    fs::create(
        Path::from("/boot").unwrap(),
        DentryType::Directory,
        DentryAttribute::Readable
            | DentryAttribute::Executable
            | DentryAttribute::PrivilegedWriteable,
    )
    .unwrap();
    for file in files {
        let path = Path::from(&format!("/boot/{}", file.filename())).unwrap();
        let parent = path.parent().unwrap();
        fs::make_directory(
            parent,
            DentryAttribute::Readable
                | DentryAttribute::Executable
                | DentryAttribute::PrivilegedWriteable,
        ).unwrap();
        fs::create_memory_stream(
            path,
            file.data(),
            DentryAttribute::Executable | DentryAttribute::Readable,
        )
        .unwrap();
        if file.filename().starts_with("bin/") {
            let process = Process::from_elf(file.data()).unwrap();
            SchedulerImpl::add(process, None);
        }
    }
    println!("\x1b[0;32m=LINK^START=\x1b[0m");
}
