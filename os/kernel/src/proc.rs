pub mod sch;

use alloc::{borrow::ToOwned, string::String, vec::Vec};
use elf_rs::{Elf, ElfFile, ElfMachine, ElfType, ProgramHeaderFlags, ProgramType};
use erhino_shared::{
    mem::{page::PageLevel, Address},
    process::{Pid, ProcessState},
};
use flagset::{flags, FlagSet};

use crate::{
    mm::{frame::frame_alloc, page::PageTableEntryFlag, unit::MemoryUnit},
    println,
    trap::TrapFrame,
};

#[derive(Debug)]
pub enum ProcessSpawnError {
    BrokenBinary,
    WrongTarget,
    InvalidPermissions
}

flags! {
    pub enum ProcessPermission: u8{
        Valid = 0b1,
        Process = 0b10,
        Memory = 0b100,
        Net = 0b1000,


        All = (ProcessPermission::Valid | ProcessPermission::Process | ProcessPermission::Memory | ProcessPermission::Net).bits()
    }
}

#[derive(Debug)]
pub struct Process {
    pub name: String,
    pub pid: Pid,
    pub parent: Pid,
    pub permissions: FlagSet<ProcessPermission>,
    pub memory: MemoryUnit,
    pub trap: TrapFrame,
    pub state: ProcessState,
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
                state: ProcessState::Ready,
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
                .fill(
                    0x03ff_fffe,
                    1,
                    PageTableEntryFlag::UserReadWrite,
                )
                .unwrap();
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


    pub fn fork<P: Into<FlagSet<ProcessPermission>>>(&mut self, permissions: P) -> Result<Process, ProcessSpawnError>{
        let perm_into: FlagSet<ProcessPermission> = permissions.into();
        let perm_new = if perm_into.contains(ProcessPermission::Valid){
            if self.permissions.contains(perm_into){
                perm_into
            }else{
                return Err(ProcessSpawnError::InvalidPermissions);
            }
        }else{
            if perm_into.is_empty(){
                self.permissions.clone()
            }else{
                return Err(ProcessSpawnError::InvalidPermissions);
            }
        };
        if self.permissions.contains(perm_into){
            let mut proc = Self{
                name: self.name.clone(),
                pid: 0,
                parent: self.pid,
                permissions: perm_new,
                memory: self.memory.fork().unwrap(),
                trap: self.trap.clone(),
                state: self.state,
                
            };
            proc.trap.satp = (8 << 60 | proc.memory.root()) as u64;
            Ok(proc)
        }else{
            Err(ProcessSpawnError::InvalidPermissions)
        }
        
    }

    pub fn has_permission(&self, perm: ProcessPermission) -> bool{
        self.permissions.contains(perm)
    }

    pub fn move_to_next_instruction(&mut self){
        self.trap.pc += 4;
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
