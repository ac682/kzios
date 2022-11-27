pub mod sch;
pub mod service;

use alloc::{borrow::ToOwned, string::String};
use elf_rs::{Elf, ElfFile, ElfMachine, ElfType, ProgramHeaderFlags, ProgramType};
use erhino_shared::{
    mem::Address,
    proc::{ExitCode, Pid, ProcessPermission, ProcessState, Signal},
};
use flagset::FlagSet;

use crate::{
    mm::{page::PageTableEntryFlag, unit::MemoryUnit},
    trap::TrapFrame,
};

#[derive(Debug)]
pub enum ProcessSpawnError {
    BrokenBinary,
    WrongTarget,
    InvalidPermissions,
}

#[derive(Clone, Copy)]
pub struct SignalControlBlock {
    pub mask: Signal,
    pub pending: Signal,
    pub handler: Address,
    pub backup: TrapFrame,
}

impl Default for SignalControlBlock {
    fn default() -> Self {
        Self {
            mask: Default::default(),
            pending: Default::default(),
            handler: Default::default(),
            backup: TrapFrame::new(),
        }
    }
}

pub struct Process {
    pub name: String,
    pub pid: Pid,
    pub parent: Pid,
    pub permissions: FlagSet<ProcessPermission>,
    pub memory: MemoryUnit,
    pub trap: TrapFrame,
    pub state: ProcessState,
    pub exit_code: ExitCode,
    signal: SignalControlBlock,
}

impl Process {
    pub fn from_elf(data: &[u8], name: &str) -> Result<Self, ProcessSpawnError> {
        if let Ok(elf) = Elf::from_bytes(data) {
            let mut process = Self {
                name: name.to_owned(),
                pid: 0,
                parent: 0,
                permissions: ProcessPermission::All.into(),
                memory: MemoryUnit::new().unwrap(),
                trap: TrapFrame::new(),
                // ignore all signal
                signal: SignalControlBlock::default(),
                state: ProcessState::Ready,
                exit_code: 0,
            };
            process.trap.pc = elf.entry_point();
            process.trap.x[2] = 0x3f_ffff_f000;
            process.trap.satp = (8 << 60) | process.memory.root() as u64;
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
                memory: self.memory.fork().unwrap(),
                trap: self.trap.clone(),
                signal: self.signal,
                state: self.state,
                exit_code: self.exit_code,
            };
            proc.trap.satp = (8 << 60 | proc.memory.root()) as u64;
            Ok(proc)
        } else {
            Err(ProcessSpawnError::InvalidPermissions)
        }
    }

    pub fn has_permission(&self, perm: ProcessPermission) -> bool {
        self.permissions.contains(perm)
    }

    pub fn move_to_next_instruction(&mut self) {
        self.trap.pc += 4;
    }

    pub fn has_signals_pending(&self) -> bool {
        self.signal.pending > 0
    }

    pub fn queue_signal(&mut self, signal: Signal) {
        self.signal.pending |= signal as Signal;
    }

    pub fn set_signal_handler(&mut self, handler: Address, mask: Signal) {
        self.signal.mask = mask;
        self.signal.handler = handler;
    }

    pub fn enter_signal(&mut self) {
        self.signal.backup = self.trap.clone();
        let mut signal = 0 as Signal;
        let mut pending = self.signal.pending;
        for i in 0..64 {
            if pending & 2 == 1 {
                signal = 1 << i;
                break;
            } else {
                pending >>= 1;
            }
        }

        self.signal.pending &= !signal;
        self.signal.backup.x[10] = signal;
        self.signal.backup.pc = self.signal.handler as u64;

        (self.trap, self.signal.backup) = (self.signal.backup, self.trap);
    }

    pub fn leave_signal(&mut self) {
        (self.trap, self.signal.backup) = (self.signal.backup, self.trap);
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
