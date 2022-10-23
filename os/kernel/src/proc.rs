pub(crate) mod pm;
pub(crate) mod sch;

use alloc::{borrow::ToOwned, string::String, vec::Vec};
use elf_rs::{Elf, ElfFile, ProgramHeaderFlags, ProgramType};
use erhino_shared::{process::ProcessState, Address, Pid};
use flagset::FlagSet;

use crate::{
    mm::{
        page::{PageTableEntryFlag},
        unit::MemoryUnit, frame::frame_alloc,
    },
    trap::TrapFrame, println,
};
use erhino_shared::page::PageLevel;

pub struct Process<'root> {
    name: String,
    pid: Pid,
    parent: Pid,
    entry_point: Address,
    memory: MemoryUnit<'root>,
    trap: TrapFrame,
    state: ProcessState,
}

pub struct ProcessTable<'root> {
    inner: Vec<Process<'root>>,
    current: usize,
}

impl<'root> Process<'root> {
    pub fn from_bytes(data: &[u8]) -> Result<Self, elf_rs::Error> {
        let elf = Elf::from_bytes(data)?;
        let mut process = Self {
            name: "any".to_owned(),
            pid: 0,
            parent: 0,
            entry_point: 0,
            memory: MemoryUnit::new(),
            trap: TrapFrame::new(),
            state: ProcessState::Ready,
        };
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
    }
}

fn flags_to_permission(
    flags: ProgramHeaderFlags,
) -> impl Into<FlagSet<PageTableEntryFlag>> + Copy {
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

impl<'root> ProcessTable<'root> {
    pub const fn new() -> Self {
        Self {
            inner: Vec::new(),
            current: 0,
        }
    }
}
