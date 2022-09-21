use core::arch::global_asm;
use core::fmt::{Debug, Formatter};
use core::ops::BitAnd;
use core::ptr::null_mut;

use elf_rs::{Elf, ElfAbi, ElfFile, ElfMachine, ElfType, ProgramHeaderFlags, ProgramType};
use flagset::FlagSet;

use crate::{_kernel_end, _kernel_start, alloc, PageTable, println};
use crate::paged::page_table::{PageTableEntry, PageTableEntryFlag};
use crate::paged::unit::MemoryUnit;
use crate::process::error::ProcessSpawnError;
use crate::trap::TrapFrame;

pub mod error;
pub mod ipc;
pub mod scheduler;
pub mod signal;

// 进程地址空间分配
const PROCESS_STACK_ADDRESS: u64 = 0x40_0000_0000 - 1; // 256GB

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
    #[no_mangle]
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
            process.memory.init(PageTable::new(2, alloc().unwrap()));
            process.trap.satp = process.memory.satp();
            process.trap.status = 1 << 7 | 1 << 5 | 1 << 4;
            let header = elf.elf_header();
            // TODO: validate RV64 from flags parameter
            if header.machine() != ElfMachine::RISC_V || header.elftype() != ElfType::ET_EXEC {
                return Err(ProcessSpawnError::WrongTarget);
            }
            for ph in elf.program_header_iter() {
                if ph.ph_type() == ProgramType::LOAD {
                    println!("map segment({:?}) {:#x}", ph.ph_type(), ph.vaddr());
                    process.memory.write(ph.vaddr(), ph.content(), Self::flags_to_permission(ph.flags()));
                }
            }
            // map stack
            process.memory.map(alloc().unwrap(), (PROCESS_STACK_ADDRESS) >> 12, PageTableEntryFlag::UserReadWrite);
            // set context
            process.trap.x[2] = PROCESS_STACK_ADDRESS;
            println!("program created at {:#x} with sp pointed to {:#x}", process.pc, process.trap.x[2]);
            process.memory.print_page_table();
            Ok(process)
        } else {
            Err(ProcessSpawnError::BrokenBinary)
        }
    }

    fn flags_to_permission(flags: ProgramHeaderFlags) -> impl Into<FlagSet<PageTableEntryFlag>> + Clone {
        let mut perm = PageTableEntryFlag::Valid | PageTableEntryFlag::User;
        let bits = flags.bits();
        if bits & 0b1 == 1 {
            perm |= PageTableEntryFlag::Executable;
        }
        if bits & 0b10 == 0b10 {
            perm |= PageTableEntryFlag::Writeable;
        }
        if bits & 0b100 == 0b100 {
            perm |= PageTableEntryFlag::Readable;
        }
        perm
    }
}
