pub mod mem;
pub mod sch;
pub mod service;
pub mod thread;

use alloc::{borrow::ToOwned, string::String};
use elf_rs::{Elf, ElfFile, ElfMachine, ElfType, ProgramHeaderFlags, ProgramType};
use erhino_shared::{
    mem::Address,
    proc::{ExitCode, Pid, ProcessPermission, ProcessState, Signal},
};
use flagset::FlagSet;

use crate::{mm::page::PageTableEntryFlag, trap::TrapFrame};

use self::mem::{unit::MemoryUnit, layout::MemoryLayout};

#[derive(Debug)]
pub enum ProcessSpawnError {
    BrokenBinary,
    WrongTarget,
    InvalidPermissions,
}

pub struct Process {
    pub name: String,
    pub pid: Pid,
    pub parent: Pid,
    pub permissions: FlagSet<ProcessPermission>,
    pub entry_point: Address,
    pub memory: MemoryUnit,
    pub layout: MemoryLayout,
    pub state: ProcessState,
    pub exit_code: ExitCode,
}

impl Process {
    pub fn from_elf(data: &[u8], name: &str) -> Result<Self, ProcessSpawnError> {
        if let Ok(elf) = Elf::from_bytes(data) {
            let mut process = Self {
                name: name.to_owned(),
                pid: 0,
                parent: 0,
                permissions: ProcessPermission::All.into(),
                entry_point: elf.entry_point() as Address,
                memory: MemoryUnit::new().unwrap(),
                layout: MemoryLayout::new(0x40_0000_0000),
                state: ProcessState::Ready,
                exit_code: 0,
            };
            let header = elf.elf_header();
            // TODO: validate RV64 from flags parameter
            if header.machine() != ElfMachine::RISC_V || header.elftype() != ElfType::ET_EXEC {
                return Err(ProcessSpawnError::WrongTarget);
            }
            process
                .memory
                .fill(0x3f_ffff_e, 1, PageTableEntryFlag::UserReadWrite)
                .unwrap();
            for ph in elf.program_header_iter() {
                if ph.ph_type() == ProgramType::LOAD {
                    process
                        .memory
                        .write(
                            ph.vaddr() as Address,
                            ph.content(),
                            ph.memsz() as usize,
                            flags_to_permission(ph.flags()),
                        )
                        .unwrap();
                }
            }
            Ok(process)
        } else {
            Err(ProcessSpawnError::BrokenBinary)
        }
    }

    pub fn fork<P: Into<FlagSet<ProcessPermission>>>(
        &mut self,
        permissions: P,
    ) -> Result<Process, ProcessSpawnError> {
        let perm_into: FlagSet<ProcessPermission> = permissions.into();
        let perm_new = if perm_into.contains(ProcessPermission::Valid) {
            if self.permissions.contains(perm_into) {
                perm_into
            } else {
                return Err(ProcessSpawnError::InvalidPermissions);
            }
        } else {
            if perm_into.is_empty() {
                self.permissions.clone()
            } else {
                return Err(ProcessSpawnError::InvalidPermissions);
            }
        };
        if self.permissions.contains(perm_into) {
            let mut proc = Self {
                name: self.name.clone(),
                pid: 0,
                parent: self.pid,
                permissions: perm_new,
                entry_point: self.entry_point.clone(),
                memory: self.memory.fork().unwrap(),
                layout: self.layout.clone(),
                state: self.state.clone(),
                exit_code: self.exit_code.clone(),
            };
            Ok(proc)
        } else {
            Err(ProcessSpawnError::InvalidPermissions)
        }
    }

    pub fn has_permission(&self, perm: ProcessPermission) -> bool {
        self.permissions.contains(perm)
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

// 4096 大小，
// 依次保存 TrapFrame 和一些用于内核数据交换的内容。由于没有监管者态，内核跑在机器态，所以不需要页表切换，也就不需要跳板页。
#[repr(C)]
pub struct KernelPage {
    trap: TrapFrame,
}
