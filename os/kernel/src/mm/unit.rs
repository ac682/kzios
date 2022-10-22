use erhino_shared::{Address, PageNumber};
use flagset::FlagSet;

use crate::{mm::range::PageRange, println};

use super::{
    frame::frame_alloc,
    page::{PageTable, PageTableEntry, PageTableEntryFlag, PageTableError},
};
use erhino_shared::page::PageLevel;

pub struct MemoryUnit<'root> {
    root: &'root mut PageTable,
}

#[derive(PartialEq, Debug, Clone, Copy)]
pub enum MemoryUnitError {
    PageNumberOutOfBound,
    RanOutOfFrames,
    EntryOverwrite,
}

impl<'root> MemoryUnit<'root> {
    pub fn new() -> Self {
        Self {
            root: PageTable::new(frame_alloc(1).unwrap()),
        }
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
                    println!("{:#x} => {:#x}", vpn + i - index, ppn + i - index);
                } else {
                    return Err(MemoryUnitError::RanOutOfFrames);
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
        } else {
            if let Some(entry) = root.entry_mut(index) {
                let branch = if entry.is_valid() {
                    entry.as_page_table()
                } else {
                    if let Some(frame) = frame_alloc(1) {
                        entry.set_as_page_table(frame)
                    } else {
                        return Err(MemoryUnitError::RanOutOfFrames);
                    }
                };
                Self::map_internal(branch, level.next_level().unwrap(), vpn, ppn, count, flags)
            } else {
                Err(MemoryUnitError::PageNumberOutOfBound)
            }
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
    pub fn write<F: Into<FlagSet<PageTableEntryFlag>>>(
        &mut self,
        addr: Address,
        data: &[u8],
        count: usize,
        flags: F,
    ) {
        todo!()
    }

    pub fn lookup(&self, addr: Address) -> Result<(PageNumber, PageLevel), PageLevel> {
        todo!()
    }

    pub fn unmap(&'root mut self, vpn: PageNumber) -> Result<(), MemoryUnitError> {
        todo!();
    }
}
