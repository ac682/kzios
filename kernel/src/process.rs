use core::arch::global_asm;
use core::fmt::{Debug, Formatter};
use core::ptr::null_mut;

use elf_rs::{Elf, ElfAbi, ElfFile, ElfMachine, ElfType, ProgramType};

use crate::paged::page_table::PageTableEntryFlags;
use crate::paged::unit::MemoryUnit;
use crate::process::error::ProcessSpawnError;
use crate::trap::TrapFrame;
use crate::{_kernel_end, _kernel_start, alloc, println, PageTable};

pub mod error;
pub mod ipc;
pub mod scheduler;
pub mod signal;

// 进程地址空间分配 for from_fn
const PROCESS_CONTROL_BLOCK_ADDRESS: u64 = 0x5000_0000 - 0x1000;
// 进程入口的前一个页
const PROCESS_ENTRY_ADDRESS: u64 = 0x4000_0000;
const PROCESS_STACK_ADDRESS: u64 = 0x8000_0000;
const PROCESS_STACK_PAGES: usize = 0x1;

#[derive(PartialEq)]
pub enum ProcessState {
    Idle,
    Running,
    Sleeping,
    Dead,
}

// 服务进程: fs, net, driver, adv_ipc
// 服务进程会一直处于 Sleeping 阶段而被跳过,当其他进程使用系统调用与服务进程通信,其会被运行并设置状态为 Running.

pub struct Process {
    trap: TrapFrame,
    pc: u64,
    pid: u64,
    // set by scheduler
    memory: MemoryUnit,
    state: ProcessState,
    exit_code: i64,
}

impl Process {
    #[deprecated]
    pub fn new_fn(func: fn()) -> Self {
        let mut process = Self {
            trap: TrapFrame::zero(),
            pc: PROCESS_ENTRY_ADDRESS + (func as u64 & 0xfff),
            pid: 0,
            memory: MemoryUnit::new(),
            state: ProcessState::Idle,
            exit_code: 0,
        };
        println!("process entry (pa): {:#x}", func as usize);
        process.memory.init(PageTable::new(2, alloc().unwrap()));
        process.trap.satp = process.memory.satp();
        process.trap.x[2] = PROCESS_STACK_ADDRESS + (PROCESS_STACK_PAGES * 4096) as u64;
        process.trap.status = 1 << 7 | 1 << 5 | 1 << 4;
        // map essential regions
        process.memory.map(
            func as u64 >> 12,
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
        process
    }

    pub fn from_elf(bytes: &[u8]) -> Result<Self, ProcessSpawnError> {
        if let Ok(elf) = Elf::from_bytes(bytes) {
            let mut process = Self {
                trap: TrapFrame::zero(),
                pc: elf.entry_point(),
                pid: 0,
                memory: MemoryUnit::new(),
                state: ProcessState::Idle,
                exit_code: 0,
            };
            let header = elf.elf_header();
            // TODO: validate RV64 from flags parameter
            if header.machine() != ElfMachine::RISC_V || header.elftype() != ElfType::ET_EXEC {
                return Err(ProcessSpawnError::WrongTarget);
            }
            for ph in elf.program_header_iter() {
                if ph.ph_type() == ProgramType::LOAD {
                    todo!("不写了把两本书看完")
                }
            }
            Ok(process)
        } else {
            Err(ProcessSpawnError::BrokenBinary)
        }
    }
}
