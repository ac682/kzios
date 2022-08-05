pub mod manager;

use crate::paged::page_table::PageTableEntryFlags;
use crate::paged::unit::MemoryUnit;
use crate::trap::TrapFrame;
use crate::{alloc, PageTable, println};
use core::arch::global_asm;
use core::ptr::null_mut;

const PROCESS_ENTRY_ADDRESS: usize = 0x5000_0000;
const PROCESS_STACK_ADDRESS: usize = 0x9000_0000;
const PROCESS_STACK_PAGES: usize = 0x1; // 4k
static mut NEXT_PID: u16 = 0;

extern "C" {
    fn _switch_to_user(frame_address: usize, pc: usize);
}

pub enum ProcessState {
    Running,
    Sleeping,
    Idle,
    Dead,
}

pub struct Process {
    trap: TrapFrame,
    stack: *mut u8,
    pc: usize,
    pid: u16,
    memory: MemoryUnit,
    state: ProcessState,
}

impl Process {
    pub fn new() -> Self {
        let mut process = Process {
            trap: TrapFrame::zero(),
            stack: null_mut(),
            pc: PROCESS_ENTRY_ADDRESS,
            pid: 0,
            memory: MemoryUnit::new(),
            state: ProcessState::Idle,
        };

        unsafe {
            process.pid = NEXT_PID;
            NEXT_PID += 1;
        }
        // here should set process.x[2] which is sp register to the right address
        // and map all pages
        process.memory.init(PageTable::new(2, alloc().unwrap()));
        // set satp
        todo!();
        process
    }

    pub fn new_fn(func: fn()) -> Self {
        let mut process = Process {
            trap: TrapFrame::zero(),
            stack: null_mut(),
            pc: PROCESS_ENTRY_ADDRESS + (func as usize & 0xfff),
            pid: 0,
            memory: MemoryUnit::new(),
            state: ProcessState::Idle,
        };
        unsafe {
            process.pid = NEXT_PID;
            NEXT_PID += 1;
        }
        println!("process entry (pa): {:#x}", func as usize);
        process.memory.init(PageTable::new(2, alloc().unwrap()));
        process.stack = PROCESS_STACK_ADDRESS as *mut u8;
        process.trap.satp = process.memory.satp();
        process.trap.x[2] = PROCESS_STACK_ADDRESS + PROCESS_STACK_PAGES * 4096;

        // map the func memory (assuming 4kb)
        process.memory.map(
            func as usize >> 12,
            PROCESS_ENTRY_ADDRESS >> 12,
            1,
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
        // map the frame context
        //process.memory.map(&process.trap as *const TrapFrame as usize >> 12, &process.trap as *const TrapFrame as usize >> 12, 1, PageTableEntryFlags::Readable | PageTableEntryFlags::Writeable);
        process
    }

    pub fn activate(&self) {
        unsafe {
            _switch_to_user(&self.trap as *const TrapFrame as usize, self.pc);
        }
    }
}
