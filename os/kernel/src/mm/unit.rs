use core::{f32::consts::E, fmt::Display, ops::Add};

use alloc::collections::BTreeMap;
use erhino_shared::mem::{page::PageLevel, Address, PageNumber};
use flagset::FlagSet;
use hashbrown::HashMap;
use spin::Once;

use crate::{
    mm::range::PageRange,
    println,
    sync::{hart::HartReadWriteLock, DataLock, InteriorLock, InteriorReadWriteLock},
};

use super::{
    frame::{frame_alloc, frame_dealloc},
    page::{PageTable, PageTableEntry, PageTableEntryFlag, PageTableError},
};

static mut TRACKED_PAGES: Once<HashMap<PageNumber, usize>> = Once::new();
static mut TRACKED_LOCK: HartReadWriteLock = HartReadWriteLock::new();

pub fn init() {
    unsafe {
        TRACKED_PAGES.call_once(|| HashMap::new());
    }
}

// 以后 MemoryUnit 可以有多种实现，例如 Sv39 可换成 Sv48
#[derive(Debug)]
pub struct MemoryUnit {
    root: &'static mut PageTable,
}

#[derive(PartialEq, Eq, Debug, Clone, Copy)]
pub enum MemoryUnitError {
    EntryNotFound,
    RanOutOfFrames,
    EntryOverwrite,
}

impl MemoryUnit {
    pub fn new() -> Result<Self, MemoryUnitError> {
        if let Some(frame) = frame_alloc(1) {
            Ok(Self {
                root: PageTable::new(frame_alloc(1).unwrap()),
            })
        } else {
            Err(MemoryUnitError::RanOutOfFrames)
        }
    }

    pub fn root(&self) -> PageNumber {
        self.root.location()
    }

    pub fn fork(&mut self) -> Result<Self, MemoryUnitError> {
        let mut unit = MemoryUnit::new()?;
        Self::copy_table(&mut self.root, &mut unit.root)?;
        Ok(unit)
    }

    fn cow_free(ppn: PageNumber) {
        let lock = unsafe { TRACKED_LOCK.lock_mut() };
        let mut tracked = unsafe { TRACKED_PAGES.get_mut().unwrap() };
        let count = tracked.get(&ppn).unwrap();
        if count == &1 {
            tracked.remove_entry(&ppn);
            frame_dealloc(ppn, 1);
        } else {
            tracked.entry(ppn).and_modify(|e| *e -= 1);
        }
    }

    fn cow_usage(ppn: PageNumber) -> usize {
        let lock = unsafe { TRACKED_LOCK.lock() };
        let mut tracked = unsafe { TRACKED_PAGES.get_unchecked() };
        let count = tracked.get(&ppn).unwrap();
        *count
    }

    pub fn handle_store_page_fault<F: Into<FlagSet<PageTableEntryFlag>> + Copy>(
        &mut self,
        addr: Address,
        flags: F,
    ) -> Result<bool, MemoryUnitError> {
        let vpn = addr >> 12;
        let entry = self.locate(vpn)?;
        if entry.is_valid() {
            if entry.is_cow_and_writeable() {
                let ppn = entry.physical_page_number();
                if Self::cow_usage(ppn) == 1 {
                    Self::cow_free(ppn);
                    entry.set(ppn, 0, entry.flags() | PageTableEntryFlag::Writeable);
                    Ok(true)
                } else {
                    if let Some(frame) = frame_alloc(1) {
                        unsafe {
                            let from = (ppn << 12) as *const Address;
                            let to = (frame << 12) as *mut Address;
                            for i in 0..4096 {
                                to.add(i).write(from.add(i).read());
                            }
                        }
                        Self::cow_free(ppn);
                        entry.set(frame, 0, entry.flags() | PageTableEntryFlag::Writeable);
                        Ok(true)
                    } else {
                        Err(MemoryUnitError::RanOutOfFrames)
                    }
                }
            } else {
                // 写入无写权限的页
                Ok(false)
            }
        } else {
            self.fill(vpn, 1, flags);
            Ok(true)
        }
    }
    fn copy_table(old: &mut PageTable, new: &mut PageTable) -> Result<(), MemoryUnitError> {
        for i in 0..512usize {
            if let Some(old_entry) = old.entry_mut(i) {
                if old_entry.is_valid() {
                    if let Some(new_entry) = new.entry_mut(i) {
                        if old_entry.is_leaf() {
                            let ppn = old_entry.physical_page_number();
                            let lock = unsafe { TRACKED_LOCK.lock_mut() };
                            let mut tracked = unsafe { TRACKED_PAGES.get_mut().unwrap() };
                            if old_entry.is_cow() {
                                tracked.entry(ppn).and_modify(|e| *e += 1).or_insert(2);
                            } else {
                                old_entry.set_cow();
                                tracked.insert(ppn, 2);
                            }
                            new_entry.write(old_entry.read());
                        } else {
                            if let Some(frame) = frame_alloc(1) {
                                Self::copy_table(
                                    old_entry.as_page_table_mut(),
                                    new_entry.set_as_page_table_mut(frame),
                                );
                            } else {
                                return Err(MemoryUnitError::RanOutOfFrames);
                            }
                        }
                    } else {
                        return Err(MemoryUnitError::EntryNotFound);
                    }
                }
            } else {
                return Err(MemoryUnitError::EntryNotFound);
            }
        }
        Ok(())
    }

    // vpn 和 ppn 都得是连续的
    pub fn map<F: Into<FlagSet<PageTableEntryFlag>> + Copy>(
        &mut self,
        vpn: PageNumber,
        ppn: PageNumber,
        count: usize,
        flags: F,
    ) -> Result<(), MemoryUnitError> {
        for i in 0..count {
            self.ensure_created(vpn + i, || Some(ppn + i), flags)?;
        }
        Ok(())
    }

    pub fn fill<F: Into<FlagSet<PageTableEntryFlag>> + Copy>(
        &mut self,
        vpn: PageNumber,
        count: usize,
        flags: F,
    ) -> Result<(), MemoryUnitError> {
        for i in 0..count {
            self.ensure_created(vpn + i, || frame_alloc(1), flags)?;
        }
        Ok(())
    }

    // 如果对应的页面没有则创建
    pub fn write<F: Into<FlagSet<PageTableEntryFlag>> + Copy>(
        &mut self,
        addr: Address,
        data: &[u8],
        length: usize,
        flags: F,
    ) -> Result<(), MemoryUnitError> {
        let real_length = if length == 0 { data.len() } else { length };
        let mut offset = addr & 0xFFF;
        let mut copied = 0usize;
        let mut page_count = 0usize;
        unsafe {
            while copied < real_length {
                let ppn =
                    self.ensure_created((addr >> 12) + page_count, || frame_alloc(1), flags)?;
                let start = (ppn << 12) + offset;
                let end = if (real_length - copied) > (0x1000 - offset as usize) {
                    (ppn + 1) << 12
                } else {
                    start + real_length - copied
                };
                let ptr = start as *mut u8;
                for i in 0..(end - start) {
                    ptr.add(i as usize)
                        .write(if copied + i as usize >= data.len() {
                            0
                        } else {
                            data[copied + i as usize]
                        });
                }
                offset = 0;
                copied += (end - start) as usize;
                page_count += 1;
            }
        }
        Ok(())
    }

    fn locate(&mut self, vpn: PageNumber) -> Result<&mut PageTableEntry, MemoryUnitError> {
        let vpn2 = PageLevel::Giga.extract(vpn);
        if let Some(entry2) = self.root.entry_mut(vpn2) {
            let table1 = if entry2.is_valid() {
                if !entry2.is_leaf() {
                    entry2.as_page_table_mut()
                } else {
                    return Err(MemoryUnitError::EntryOverwrite);
                }
            } else if let Some(frame) = frame_alloc(1) {
                entry2.set_as_page_table_mut(frame)
            } else {
                return Err(MemoryUnitError::RanOutOfFrames);
            };
            let vpn1 = PageLevel::Mega.extract(vpn);
            if let Some(entry1) = table1.entry_mut(vpn1) {
                let table0 = if entry1.is_valid() {
                    if !entry1.is_leaf() {
                        entry1.as_page_table_mut()
                    } else {
                        return Err(MemoryUnitError::EntryOverwrite);
                    }
                } else if let Some(frame) = frame_alloc(1) {
                    entry1.set_as_page_table_mut(frame)
                } else {
                    return Err(MemoryUnitError::RanOutOfFrames);
                };
                let vpn0 = PageLevel::Kilo.extract(vpn);
                if let Some(entry0) = table0.entry_mut(vpn0) {
                    Ok(entry0)
                } else {
                    Err(MemoryUnitError::EntryNotFound)
                }
            } else {
                Err(MemoryUnitError::EntryNotFound)
            }
        } else {
            Err(MemoryUnitError::EntryNotFound)
        }
    }

    fn ensure_created<
        F: Into<FlagSet<PageTableEntryFlag>> + Copy,
        T: Fn() -> Option<PageNumber>,
    >(
        &mut self,
        vpn: PageNumber,
        ppn_factory: T,
        flags: F,
    ) -> Result<PageNumber, MemoryUnitError> {
        let entry = self.locate(vpn)?;
        if entry.is_valid() {
            if entry.is_leaf() {
                let bits = entry.flags().bits();
                let new_bits = flags.into().bits();
                if bits != new_bits {
                    entry.write_bitor(new_bits);
                }
                Ok(entry.physical_page_number())
            } else {
                Err(MemoryUnitError::EntryOverwrite)
            }
        } else if let Some(ppn) = ppn_factory() {
            entry.set(ppn, 0, flags);
            Ok(ppn)
        } else {
            Err(MemoryUnitError::RanOutOfFrames)
        }
    }

    pub fn lookup(&self, addr: Address) -> Result<(PageNumber, PageLevel), PageLevel> {
        todo!()
    }

    pub fn unmap(&mut self, vpn: PageNumber) -> Result<(), MemoryUnitError> {
        todo!();
    }
}

impl Display for MemoryUnit {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        writeln!(f, "memory@{:#x} {{", self.root.location())?;
        writeln!(f, "                                DAGUXWRV")?;
        let table2 = &self.root;
        for vpn2 in 0..512 {
            if let Some(entry2) = table2.entry(vpn2) {
                if entry2.is_valid() && !entry2.is_leaf() {
                    let table1 = entry2.as_page_table();
                    for vpn1 in 0..512 {
                        if let Some(entry1) = table1.entry(vpn1) {
                            if entry1.is_valid() && !entry1.is_leaf() {
                                let table0 = entry1.as_page_table();
                                for vpn0 in 0..512 {
                                    if let Some(entry0) = table0.entry(vpn0) {
                                        if entry0.is_valid() && entry0.is_leaf() {
                                            writeln!(
                                                f,
                                                "{:#12x} => {:#12x} ({:#010b}) ",
                                                (vpn2 << 18) | (vpn1 << 9) | vpn0,
                                                entry0.physical_page_number(),
                                                entry0.flags().bits()
                                            )?;
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
        writeln!(f, "}}")
    }
}
