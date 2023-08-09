use core::sync::atomic::{AtomicU32, AtomicUsize, Ordering};

use alloc::{borrow::ToOwned, string::String};
use elf_rs::{Elf, ElfFile, ElfMachine, ElfType, ProgramType};
use erhino_shared::{
    mem::Address,
    proc::{ExecutionState, ExitCode, Pid, ProcessPermission, Tid},
};
use flagset::FlagSet;

use crate::{
    external::_trampoline,
    mm::{
        page::{PageFlag, PageTableEntry, PageTableEntry39},
        unit::MemoryUnit,
    },
};

use super::thread::Thread;

type PageEntryImpl = PageTableEntry39;

static PID_GENERATOR: AtomicU32 = AtomicU32::new(0);

#[derive(Debug)]
pub enum ProcessSpawnError {
    BrokenBinary,
    WrongTarget,
    InvalidPermissions,
}

pub struct Process {
    pub pid: Pid,
    pub parent: Pid,
    pub name: String,
    pub memory: MemoryUnit<PageEntryImpl>,
    pub entry_point: Address,
    pub permissions: FlagSet<ProcessPermission>,
    pub exit_code: ExitCode,
    tid_generator: AtomicU32,
}

impl Process {
    fn next_pid() -> Pid {
        PID_GENERATOR.fetch_add(1, Ordering::Relaxed) as Pid
    }

    fn next_tid(&self) -> Tid {
        self.tid_generator.fetch_add(1, Ordering::Relaxed) as Tid
    }

    pub fn from_elf(data: &[u8], name: &str) -> Result<Self, ProcessSpawnError> {
        if let Ok(elf) = Elf::from_bytes(data) {
            let top = 1usize << ((PageEntryImpl::DEPTH * PageEntryImpl::SIZE + 12) - 1);
            let pid = Self::next_pid();
            let mut process = Self {
                name: name.to_owned(),
                pid: pid,
                parent: pid,
                permissions: ProcessPermission::All.into(),
                entry_point: elf.entry_point() as Address,
                memory: MemoryUnit::new().unwrap(),
                // layout: MemoryLayout::new(top),
                exit_code: 0,
                tid_generator: AtomicU32::new(0),
            };
            let header = elf.elf_header();
            // TODO: validate RV64 from flags parameter
            if header.machine() != ElfMachine::RISC_V || header.elftype() != ElfType::ET_EXEC {
                return Err(ProcessSpawnError::WrongTarget);
            }
            // process
            //     .memory
            //     .fill(0x3f_ffff_e, 1, PageFlag::UserReadWrite)
            //     .unwrap();
            for ph in elf.program_header_iter() {
                // if ph.ph_type() == ProgramType::LOAD {
                //     process
                //         .memory
                //         .write(
                //             ph.vaddr() as Address,
                //             ph.content(),
                //             ph.memsz() as usize,
                //             flags_to_permission(ph.flags()),
                //         )
                //         .unwrap();
                // }
            }
            let top = MemoryUnit::<PageEntryImpl>::top_page_number();
            process.memory.map(
                top,
                _trampoline as usize >> 12,
                1,
                PageFlag::Valid
                    | PageFlag::Readable
                    | PageFlag::Writeable
                    | PageFlag::Executable
                    | PageFlag::User,
            );
            // process.memory.fill(top - 1, 1, PageFlag::Valid
            //     | PageFlag::Readable
            //     | PageFlag::Writeable
            //     | PageFlag::User);
            Ok(process)
        } else {
            Err(ProcessSpawnError::BrokenBinary)
        }
    }

    pub fn spawn(&self) -> Thread {
        todo!()
    }
}
