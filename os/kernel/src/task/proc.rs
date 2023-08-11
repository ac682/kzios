use core::sync::atomic::{AtomicU32, AtomicUsize, Ordering};

use alloc::{borrow::ToOwned, string::String};
use elf_rs::{Elf, ElfFile, ElfMachine, ElfType, ProgramType, ProgramHeaderFlags};
use erhino_shared::{
    mem::Address,
    proc::{ExecutionState, ExitCode, Pid, ProcessPermission, Tid},
};
use flagset::FlagSet;

use crate::mm::{
        page::{PageEntryFlag, PageTableEntry, PageTableEntry39, PageEntryImpl},
        unit::MemoryUnit,
    };

use super::thread::Thread;

#[derive(Debug)]
pub enum ProcessSpawnError {
    BrokenBinary,
    WrongTarget,
    InvalidPermissions,
}

pub struct Process {
    pub name: String,
    pub memory: MemoryUnit<PageEntryImpl>,
    pub entry_point: Address,
    pub permissions: FlagSet<ProcessPermission>,
}

impl Process {

    pub fn from_elf(data: &[u8], name: &str) -> Result<Self, ProcessSpawnError> {
        if let Ok(elf) = Elf::from_bytes(data) {
            let top = 1usize << ((PageEntryImpl::DEPTH * PageEntryImpl::SIZE + 12) - 1);
            let mut process = Self {
                name: name.to_owned(),
                permissions: ProcessPermission::All.into(),
                entry_point: elf.entry_point() as Address,
                memory: MemoryUnit::new().unwrap(),
                // layout: MemoryLayout::new(top),
            };
            let header = elf.elf_header();
            // TODO: validate RV64 from flags parameter
            if header.machine() != ElfMachine::RISC_V || header.elftype() != ElfType::ET_EXEC {
                return Err(ProcessSpawnError::WrongTarget);
            }
            // process
            //     .memory
            //     .fill(0x3f_ffff_e, 1, PageEntryFlag::UserReadWrite)
            //     .unwrap();
            for ph in elf.program_header_iter() {
                if ph.ph_type() == ProgramType::LOAD {
                    if let Some(content) = ph.content(){
                        process
                        .memory
                        .write(
                            ph.vaddr() as Address,
                            content,
                            ph.memsz() as usize,
                            flags_to_permission(ph.flags()),
                        )
                        .unwrap();
                    }
                }
            }
            Ok(process)
        } else {
            Err(ProcessSpawnError::BrokenBinary)
        }
    }
}

fn flags_to_permission(flags: ProgramHeaderFlags) -> impl Into<FlagSet<PageEntryFlag>> + Copy {
    let mut perm = PageEntryFlag::Valid | PageEntryFlag::User;
    let bits = flags.bits();
    if bits & 0b1 == 1 {
        perm |= PageEntryFlag::Executable;
    }
    if bits & 0b10 == 0b10 {
        perm |= PageEntryFlag::Writeable;
    }
    if bits & 0b100 == 0b100 {
        perm |= PageEntryFlag::Readable;
    }
    perm
}