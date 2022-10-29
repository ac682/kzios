pub mod sch;

use alloc::{borrow::ToOwned, string::String, vec::Vec};
use elf_rs::{Elf, ElfFile, ElfMachine, ElfType, ProgramHeaderFlags, ProgramType};
use erhino_shared::{process::{ProcessState, Pid}, mem::{Address, page::PageLevel}};
use flagset::FlagSet;

use crate::{
    mm::{frame::frame_alloc, page::PageTableEntryFlag, unit::MemoryUnit},
    println,
    trap::TrapFrame,
};

#[derive(Debug)]
pub enum ProcessSpawnError {
    BrokenBinary,
    WrongTarget,
}

#[derive(Debug)]
pub struct Process {
    pub name: String,
    pub pid: Pid,
    pub parent: Pid,
    pub memory: MemoryUnit,
    pub trap: TrapFrame,
    pub state: ProcessState,
}

impl Process {
    pub fn from_elf(data: &[u8]) -> Result<Self, ProcessSpawnError> {
        if let Ok(elf) = Elf::from_bytes(data) {
            let mut process = Self {
                name: "adam".to_owned(),
                pid: 0,
                parent: 0,
                memory: MemoryUnit::new(),
                trap: TrapFrame::new(),
                state: ProcessState::Ready,
            };
            process.trap.pc = elf.entry_point();
            process.trap.x[2] = 0x40_0000_0000;
            process.trap.satp = (8 << 60) | process.memory.root() as u64;
            process.trap.status = 1 << 13 | 1 << 7 | 1 << 5 | 1 << 4;
            let header = elf.elf_header();
            // TODO: validate RV64 from flags parameter
            if header.machine() != ElfMachine::RISC_V || header.elftype() != ElfType::ET_EXEC {
                return Err(ProcessSpawnError::WrongTarget);
            }
            process.memory.fill(0x3f_ffff_f, 1, PageTableEntryFlag::UserReadWrite | PageTableEntryFlag::Valid).unwrap();
            for ph in elf.program_header_iter() {
                if ph.ph_type() == ProgramType::LOAD {
                    process.memory.write(
                            ph.vaddr() as Address,
                    ph.content(),
                    ph.memsz() as usize,
                    flags_to_permission(ph.flags()),
                    );
                }
            }
            Ok(process)
        } else {
            Err(ProcessSpawnError::BrokenBinary)
        }
    }
}

fn flags_to_permission(flags: ProgramHeaderFlags) -> impl Into<FlagSet<PageTableEntryFlag>> + Copy {
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