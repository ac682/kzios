use core::arch::global_asm;
use core::fmt::{Debug, Formatter};
use core::ptr::null_mut;

use crate::paged::page_table::PageTableEntryFlags;
use crate::paged::unit::MemoryUnit;
use crate::trap::TrapFrame;
use crate::{_kernel_end, _kernel_start, alloc, println, PageTable};

pub mod scheduler;

const PROCESS_ENTRY_ADDRESS: usize = 0x5000_0000;
const PROCESS_STACK_ADDRESS: usize = 0x9000_0000;
const PROCESS_STACK_PAGES: usize = 0x1;

#[derive(PartialEq)]
pub enum ProcessState {
    Running,
    Sleeping,
    Idle,
    Dead,
}

pub struct Process {
    trap: TrapFrame,
    pc: usize,
    pid: usize,
    // set by scheduler
    memory: MemoryUnit,
    state: ProcessState,
    exit_code: u32,
}

impl Process {
    pub fn new_fn(func: fn()) -> Self {
        let mut process = Process {
            trap: TrapFrame::zero(),
            pc: PROCESS_ENTRY_ADDRESS + (func as usize & 0xfff),
            pid: 0,
            memory: MemoryUnit::new(),
            state: ProcessState::Idle,
            exit_code: 0,
        };
        println!("process entry (pa): {:#x}", func as usize);
        process.memory.init(PageTable::new(2, alloc().unwrap()));
        process.trap.satp = process.memory.satp();
        process.trap.x[2] = PROCESS_STACK_ADDRESS + PROCESS_STACK_PAGES * 4096;
        process.trap.status = 1 << 7 | 1 << 5 | 1 << 4;
        // map essential regions
        // map the kernel
        //process.memory.map(_kernel_start as usize, _kernel_start as usize, (_kernel_end as usize - _kernel_start as usize) >> 12, PageTableEntryFlags::User | PageTableEntryFlags::Readable | PageTableEntryFlags::Executable);
        // map the func memory (assuming 4kb)
        process.memory.map(
            func as usize >> 12,
            PROCESS_ENTRY_ADDRESS >> 12,
            2,
            PageTableEntryFlags::UserReadWrite | PageTableEntryFlags::Executable,
        );
        // map the stack
        process.memory.fill(
            || alloc().unwrap(),
            PROCESS_STACK_ADDRESS >> 12,
            PROCESS_STACK_PAGES,
            PageTableEntryFlags::UserReadWrite,
        );
        process.memory.print_page_table();
        process
    }
}
