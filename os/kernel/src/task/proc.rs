use alloc::{borrow::ToOwned, string::String, vec::Vec};
use elf_rs::{Elf, ElfFile, ElfMachine, ElfType, ProgramHeaderFlags, ProgramType};
use erhino_shared::{
    mem::{Address, MemoryRegionAttribute, PageNumber},
    proc::{ProcessPermission, ExitCode},
};
use flagset::FlagSet;

use crate::mm::{
        page::{PageEntryFlag, PageEntryImpl, PAGE_BITS, PAGE_SIZE},
        unit::{MemoryUnit, MemoryUnitError},
        usage::MemoryUsage,
    };

#[allow(unused)]
#[derive(Debug)]
pub enum ProcessSpawnError {
    BrokenBinary,
    WrongTarget,
    InvalidPermissions,
}

#[derive(Debug)]
pub enum ProcessMemoryError {
    Unknown,
    ConflictingMapping,
    MisalignedAddress,
    OutOfMemory,
    InaccessibleRegion,
}

impl From<MemoryUnitError> for ProcessMemoryError {
    fn from(value: MemoryUnitError) -> Self {
        match value {
            MemoryUnitError::EntryNotFound => Self::Unknown,
            MemoryUnitError::EntryOverwrite => Self::ConflictingMapping,
            MemoryUnitError::RanOutOfFrames => Self::OutOfMemory,
        }
    }
}

#[derive(Debug, PartialEq, Eq)]
pub enum ProcessHealth{
    Healthy,
    Dead(ExitCode)
}

pub struct Process {
    pub name: String,
    memory: MemoryUnit<PageEntryImpl>,
    pub usage: MemoryUsage,
    pub entry_point: Address,
    pub break_point: Address,
    pub permissions: FlagSet<ProcessPermission>,
    pub health: ProcessHealth
}

impl Process {
    pub fn from_elf(data: &[u8], name: &str) -> Result<Self, ProcessSpawnError> {
        if let Ok(elf) = Elf::from_bytes(data) {
            let mut process = Self {
                name: name.to_owned(),
                permissions: ProcessPermission::All.into(),
                entry_point: elf.entry_point() as Address,
                break_point: 0,
                memory: MemoryUnit::new().unwrap(),
                usage: MemoryUsage::new(),
                health: ProcessHealth::Healthy
            };
            let header = elf.elf_header();
            if header.machine() != ElfMachine::RISC_V || header.elftype() != ElfType::ET_EXEC {
                return Err(ProcessSpawnError::WrongTarget);
            }
            let mut max_addr = 0usize;
            let mut byte_used = 0usize;
            let mut page_used = 0usize;
            for ph in elf.program_header_iter() {
                if ph.ph_type() == ProgramType::LOAD {
                    let addr = ph.vaddr() as usize;
                    if let Some(content) = ph.content() {
                        let length = ph.memsz() as usize;
                        let vpn = addr >> PAGE_BITS;
                        let attr = flags_to_attrs(ph.flags());
                        process
                            .fill(
                                vpn,
                                ((addr + length + PAGE_SIZE - 1) >> PAGE_BITS) - vpn,
                                attr,
                                false,
                            )
                            .map(|w| page_used += w)
                            .expect("create program segment space failed");
                        process
                            .write(addr as Address, content, length)
                            .map(|w| byte_used += w)
                            .expect("write program segment to its process space failed");
                    }
                    if addr > max_addr {
                        max_addr = addr;
                    }
                }
            }
            let brk = (max_addr as usize + PAGE_SIZE - 1) & !(PAGE_SIZE - 1);
            process.usage.program = byte_used as usize;
            process.usage.page = page_used;
            process.break_point = brk;
            Ok(process)
        } else {
            Err(ProcessSpawnError::BrokenBinary)
        }
    }

    pub fn fill<A: Into<FlagSet<MemoryRegionAttribute>>>(
        &mut self,
        vpn: PageNumber,
        count: usize,
        attributes: A,
        reserved: bool,
    ) -> Result<usize, ProcessMemoryError> {
        let flags = attrs_to_flags(attributes, reserved);
        self.memory
            .fill(vpn, count, flags)
            .map(|p| {
                self.usage.page += p;
                p
            })
            .map_err(|e| ProcessMemoryError::from(e))
    }

    pub fn map<A: Into<FlagSet<MemoryRegionAttribute>>>(
        &mut self,
        vpn: PageNumber,
        ppn: PageNumber,
        count: usize,
        attributes: A,
        reserved: bool,
    ) -> Result<usize, ProcessMemoryError> {
        let flags = attrs_to_flags(attributes, reserved);
        self.memory
            .map(vpn, ppn, count, flags)
            .map(|p| {
                self.usage.page += p;
                p
            })
            .map_err(|e| ProcessMemoryError::from(e))
    }

    pub fn extend(&mut self, size: usize) -> Result<usize, ProcessMemoryError> {
        if !size.is_power_of_two() {
            return Err(ProcessMemoryError::MisalignedAddress);
        }
        let start = self.break_point + self.usage.heap;
        let count = (size + PAGE_SIZE - 1) >> PAGE_BITS;
        let flags = attrs_to_flags(
            MemoryRegionAttribute::Write | MemoryRegionAttribute::Read,
            false,
        );
        match self.memory.fill(start >> PAGE_BITS, count, flags) {
            Ok(pages) => {
                self.usage.page += pages;
                self.usage.heap += size;
                Ok(start + size)
            }
            Err(error) => Err(ProcessMemoryError::from(error)),
        }
    }

    pub fn write(
        &mut self,
        address: Address,
        data: &[u8],
        length: usize,
    ) -> Result<usize, ProcessMemoryError> {
        let real_length = if length == 0 { data.len() } else { length };
        let mut written = 0usize;
        while written < real_length {
            if let Some(base) = self.translate(address + written) {
                let offset = base & (PAGE_SIZE - 1);
                let start = base as *mut u8;
                let space = PAGE_SIZE - offset;
                let count = if real_length - written > space {
                    space
                } else {
                    real_length - written
                };
                for i in 0..count {
                    unsafe {
                        start.add(i).write(if written + i >= data.len() {
                            0
                        } else {
                            data[written + i]
                        });
                    }
                }
                written += count;
            } else {
                return Err(ProcessMemoryError::InaccessibleRegion);
            }
        }
        Ok(written)
    }

    pub fn read(&self, address: Address, length: usize) -> Result<Vec<u8>, ProcessMemoryError> {
        let mut container = Vec::<u8>::with_capacity(length);
        let mut read = 0usize;
        while read < length {
            if let Some(base) = self.translate(address + read) {
                let offset = base & (PAGE_SIZE - 1);
                let start = (base + offset) as *const u8;
                let space = PAGE_SIZE - offset;
                let count = if length - read > space {
                    space
                } else {
                    length - read
                };
                for i in 0..count {
                    container.push(unsafe { start.add(i).read() });
                }
                read += count;
            } else {
                return Err(ProcessMemoryError::InaccessibleRegion);
            }
        }
        Ok(container)
    }

    pub fn translate(&self, address: Address) -> Option<Address> {
        self.memory.translate(address).map(|(a, _)| a)
    }

    pub fn page_table_token(&self) -> usize {
        self.memory.satp()
    }

    pub fn usage(&self) -> &MemoryUsage {
        &self.usage
    }
}

fn attrs_to_flags<A: Into<FlagSet<MemoryRegionAttribute>>>(
    attributes: A,
    reserved: bool,
) -> FlagSet<PageEntryFlag> {
    let mut flags: FlagSet<PageEntryFlag> = PageEntryFlag::Valid.into();
    let attr: FlagSet<MemoryRegionAttribute> = attributes.into();
    if attr.contains(MemoryRegionAttribute::Read) {
        flags |= PageEntryFlag::Readable;
    }
    if attr.contains(MemoryRegionAttribute::Write) {
        flags |= PageEntryFlag::Writeable;
    }
    if attr.contains(MemoryRegionAttribute::Execute) {
        flags |= PageEntryFlag::Executable;
    }
    if !reserved {
        flags |= PageEntryFlag::User;
    }
    flags
}

fn flags_to_attrs(flags: ProgramHeaderFlags) -> FlagSet<MemoryRegionAttribute> {
    let mut attr: FlagSet<MemoryRegionAttribute> = MemoryRegionAttribute::None.into();
    if flags.contains(ProgramHeaderFlags::EXECUTE) {
        attr |= MemoryRegionAttribute::Execute;
    }
    if flags.contains(ProgramHeaderFlags::WRITE) {
        attr |= MemoryRegionAttribute::Write;
    }
    if flags.contains(ProgramHeaderFlags::READ) {
        attr |= MemoryRegionAttribute::Read;
    }
    attr
}
