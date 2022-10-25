use erhino_shared::{Address, PageNumber};
use flagset::FlagSet;

use crate::{mm::range::PageRange, println};

use super::{
    frame::frame_alloc,
    page::{PageTable, PageTableEntry, PageTableEntryFlag, PageTableError},
};
use erhino_shared::page::PageLevel;

#[derive(Debug)]
pub struct MemoryUnit {
    root: &'static mut PageTable,
}

#[derive(PartialEq, Debug, Clone, Copy)]
pub enum MemoryUnitError {
    EntryNotFound,
    RanOutOfFrames,
    EntryOverwrite,
}

impl MemoryUnit {
    pub fn new() -> Self {
        Self {
            root: PageTable::new(frame_alloc(1).unwrap()),
        }
    }

    pub fn root(&self) -> PageNumber {
        self.root.location()
    }

    // vpn 和 ppn 都得是连续的
    pub fn map<F: Into<FlagSet<PageTableEntryFlag>> + Copy>(
        &mut self,
        vpn: PageNumber,
        ppn: PageNumber,
        count: usize,
        flags: F,
    ) -> Result<(), MemoryUnitError> {
        Self::map_internal(self.root, PageLevel::Giga, vpn, ppn, count, flags)
    }

    fn map_internal<F: Into<FlagSet<PageTableEntryFlag>> + Copy>(
        root: &mut PageTable,
        level: PageLevel,
        vpn: PageNumber,
        ppn: PageNumber,
        count: usize,
        flags: F,
    ) -> Result<(), MemoryUnitError> {
        let index = level.extract(vpn);
        if PageLevel::Kilo == level {
            let end = if index + count < 512 { count } else { 512 };
            for i in index..end {
                if let Some(entry) = root.entry_mut(i) {
                    entry.set(ppn + i - index, 0, flags);
                } else {
                    return Err(MemoryUnitError::EntryNotFound);
                }
            }
            let mapped = end - index;
            if count > 512 - index {
                Self::map_internal(
                    root,
                    level,
                    vpn + mapped,
                    ppn + mapped,
                    count - mapped,
                    flags,
                )
            } else {
                Ok(())
            }
        } else if let Some(entry) = root.entry_mut(index) {
            let branch = if entry.is_valid() {
                entry.as_page_table()
            } else if let Some(frame) = frame_alloc(1) {
                entry.set_as_page_table(frame)
            } else {
                return Err(MemoryUnitError::RanOutOfFrames);
            };
            // 如果跨表发生的很频繁，分配的页很多的话，会爆栈。但是未来会改成性能更好的多类型页分配的对吧
            // 爆个屁，这是尾递归，编译器一定会优化的，一定会
            Self::map_internal(branch, level.next_level().unwrap(), vpn, ppn, count, flags)
        } else {
            Err(MemoryUnitError::EntryNotFound)
        }
    }

    pub fn fill<F: Into<FlagSet<PageTableEntryFlag>> + Copy>(
        &mut self,
        vpn: PageNumber,
        count: usize,
        flags: F,
    ) {
        todo!()
    }

    // 如果对应的页面没有则创建
    pub fn write<F: Into<FlagSet<PageTableEntryFlag>> + Copy>(
        &mut self,
        addr: Address,
        data: &[u8],
        count: usize,
        flags: F,
    ) -> Result<(), MemoryUnitError> {
        Self::write_one_page_once_then_next(self.root, PageLevel::Giga, addr, 0, data, count, flags)
    }

    fn write_one_page_once_then_next<F: Into<FlagSet<PageTableEntryFlag>> + Copy>(
        root: &mut PageTable,
        level: PageLevel,
        addr: Address,
        offset: usize,
        data: &[u8],
        count: usize,
        flags: F,
    ) -> Result<(), MemoryUnitError> {
        let vpn = addr >> 12;
        let index = level.extract(vpn);
        if PageLevel::Kilo == level {
            if let Some(entry) = root.entry_mut(index) {
                let ppn = if entry.is_leaf() && entry.is_valid() {
                    let f = flags.into();
                    if entry.flags().bits() != f.bits() {
                        entry.write_bitor(f.bits());
                    }
                    entry.physical_page_number()
                } else if let Some(frame) = frame_alloc(1) {
                    entry.set(frame, 0, flags);
                    frame
                } else {
                    return Err(MemoryUnitError::RanOutOfFrames);
                };
                let paddr = (ppn << 12) + (addr & 0xfff);
                let ptr = paddr as *mut u8;
                let remaining_space = PageLevel::Kilo.size_of_bytes() - (addr & 0xfff);
                let needed = if count > remaining_space {
                    remaining_space
                } else {
                    count
                };
                unsafe {
                    for i in 0..needed {
                        ptr.add(i).write(if (i + offset) < data.len() {
                            data[i + offset]
                        } else {
                            0
                        });
                    }
                }
                if remaining_space < count {
                    Self::write_one_page_once_then_next(
                        root,
                        level,
                        (vpn + 1) << 12,
                        needed,
                        data,
                        count - remaining_space,
                        flags,
                    )
                } else {
                    Ok(())
                }
            } else {
                Err(MemoryUnitError::EntryNotFound)
            }
        } else if let Some(entry) = root.entry_mut(index) {
            let branch = if entry.is_valid() {
                entry.as_page_table()
            } else if let Some(frame) = frame_alloc(1) {
                entry.set_as_page_table(frame)
            } else {
                return Err(MemoryUnitError::RanOutOfFrames);
            };
            Self::write_one_page_once_then_next(
                branch,
                level.next_level().unwrap(),
                addr,
                offset,
                data,
                count,
                flags,
            )
        } else {
            Err(MemoryUnitError::EntryNotFound)
        }
    }

    pub fn lookup(&self, addr: Address) -> Result<(PageNumber, PageLevel), PageLevel> {
        todo!()
    }

    pub fn unmap(&mut self, vpn: PageNumber) -> Result<(), MemoryUnitError> {
        todo!();
    }
}
