// Sv39 only

use core::{cell::UnsafeCell, fmt::Debug, mem::size_of, ops::Not};

use alloc::boxed::Box;
use erhino_shared::mem::{Address, PageNumber};
use flagset::{flags, FlagSet};
use hashbrown::HashMap;
use num_traits::Pow;

use super::frame::FrameTracker;

const PAGE_SIZE: usize = 4096;

flags! {
    pub enum PageFlag: u64{
        Valid = 0b1,
        Readable = 0b10,
        Writeable = 0b100,
        Executable = 0b1000,
        User = 0b1_0000,
        Global = 0b10_0000,
        Accessed = 0b100_0000,
        Dirty = 0b1000_0000,
        Cow = 0b1_0000_0000,
        CowWriteable = 0b10_0000_0000,

        UserReadWrite = (PageFlag::User | PageFlag::Readable | PageFlag::Writeable | PageFlag::Valid).bits(),
    }
}

pub struct PageTable<E: PageTableEntry + 'static> {
    location: PageNumber,
    entries: &'static mut [E],
    branches: HashMap<usize, PageTable<E>>,
    // PageTable 必定 managed， leaves 有可能 managed
    managed: HashMap<usize, FrameTracker>,
}

impl<E: PageTableEntry + 'static> PageTable<E> {
    pub fn from(location: PageNumber) -> Self {
        let entries = unsafe {
            core::slice::from_raw_parts_mut((location << 12) as *mut E, PAGE_SIZE / size_of::<E>())
                .try_into()
                .unwrap()
        };
        Self {
            location,
            entries: entries,
            branches: HashMap::new(),
            managed: HashMap::new(),
        }
    }

    pub fn page_number(&self) -> PageNumber {
        self.location
    }

    fn entry(&self, index: usize) -> &E {
        &self.entries[index]
    }

    fn entry_mut(&mut self, index: usize) -> &mut E {
        &mut self.entries[index]
    }

    pub fn is_entry_created(&self, index: usize) -> bool {
        self.entry(index).is_valid()
    }

    pub fn create_leaf<F: Into<FlagSet<PageFlag>>>(
        &mut self,
        index: usize,
        ppn: PageNumber,
        flags: F,
    ) -> Option<&mut E> {
        let entry = self.entry_mut(index);
        if !entry.is_valid() {
            entry.set(ppn, flags);
            Some(entry)
        } else {
            None
        }
    }

    // return frame tracker if failed
    pub fn create_managed_leaf<F: Into<FlagSet<PageFlag>>>(
        &mut self,
        index: usize,
        tracker: FrameTracker,
        flags: F,
    ) -> Option<FrameTracker> {
        let entry = self.entry_mut(index);
        let ppn = tracker.start();
        if !entry.is_valid() {
            entry.set(ppn, flags);
            self.managed.insert(index, tracker);
            None
        } else {
            Some(tracker)
        }
    }

    pub fn get_table(&self, index: usize) -> Option<&PageTable<E>> {
        let entry = self.entry(index);
        if entry.is_valid() && !entry.is_leaf() {
            self.branches.get(&index)
        } else {
            None
        }
    }

    pub fn ensure_table_created<F: Fn() -> Option<FrameTracker>>(
        &mut self,
        index: usize,
        frame_factory: F,
    ) -> Option<&mut PageTable<E>> {
        let entry = self.entry_mut(index);

        if entry.is_valid() {
            if !entry.is_leaf() {
                self.branches.get_mut(&index)
            } else {
                None
            }
        } else {
            if let Some(frame) = frame_factory() {
                let ppn = frame.start();
                entry.set(ppn, PageFlag::Valid);
                let table = PageTable::<E>::from(ppn);
                self.managed.insert(index, frame);
                self.branches.insert(index, table);
                self.branches.get_mut(&index)
            } else {
                None
            }
        }
    }

    pub const fn entry_count() -> usize {
        PAGE_SIZE / size_of::<E>()
    }
}

pub trait PageTableEntry: Sized {
    const LENGTH: usize;
    const DEPTH: usize;
    const SIZE: usize;
    fn space_size() -> usize;
    fn top_address() -> Address;
    fn is_leaf(&self) -> bool;
    fn is_valid(&self) -> bool;
    fn has_flag(&self, flag: PageFlag) -> bool;
    fn set_flag(&mut self, flag: PageFlag);
    fn clear_flag(&mut self, flag: PageFlag);
    fn flags(&self) -> FlagSet<PageFlag>;
    fn physical_page_number(&self) -> PageNumber;
    fn set<F: Into<FlagSet<PageFlag>>>(&mut self, ppn: PageNumber, flags: F);
}

pub struct PageTableEntryPrimitive<
    P: Clone + Copy + Into<u64> + TryFrom<u64> + From<u8> + Not,
    const LENGTH: usize,
    const DEPTH: usize,
    const SIZE: usize,
>(P);

impl<
        P: Clone + Copy + Into<u64> + TryFrom<u64> + From<u8> + Not,
        const LENGTH: usize,
        const DEPTH: usize,
        const SIZE: usize,
    > PageTableEntry for PageTableEntryPrimitive<P, LENGTH, DEPTH, SIZE>
{
    const LENGTH: usize = LENGTH;

    const DEPTH: usize = DEPTH;

    const SIZE: usize = SIZE;

    fn space_size() -> usize {
        1usize << ((DEPTH * SIZE + 12) - 1)
    }
    // 以 Sv39 为例，其虚拟地址空间大小为 2^64
    // 但是 有效位为 (0){25}0(x){38} 或 (1){25}1(x){38}
    // 空间大小为 2^38，为上下各 0+2^38 和 2^64-2^38
    fn top_address() -> Address {
        let zero: u64 = P::from(0u8).into();
        !zero as Address
    }

    fn is_leaf(&self) -> bool {
        self.0.into() & 0b1110 != 0
    }

    fn is_valid(&self) -> bool {
        self.0.into() & 0b1 != 0
    }

    fn set_flag(&mut self, flag: PageFlag) {
        let pre = self.0.into() | flag as u64;
        if let Ok(p) = P::try_from(pre) {
            self.0 = p
        }
    }

    fn clear_flag(&mut self, flag: PageFlag) {
        let pre = self.0.into() & (!0u64 - flag as u64);
        if let Ok(p) = P::try_from(pre) {
            self.0 = p
        }
    }

    fn has_flag(&self, flag: PageFlag) -> bool {
        self.0.into() & flag as u64 != 0
    }

    fn flags(&self) -> FlagSet<PageFlag> {
        FlagSet::new(self.0.into() & FlagSet::<PageFlag>::full().bits()).unwrap()
    }

    fn physical_page_number(&self) -> PageNumber {
        ((self.0.into() & ((1 << Self::LENGTH) - 1)) >> 10) as PageNumber
    }

    fn set<F: Into<FlagSet<PageFlag>>>(&mut self, ppn: PageNumber, flags: F) {
        let pre = ((ppn as u64) << 10) + flags.into().bits();
        if let Ok(p) = P::try_from(pre) {
            self.0 = p
        }
    }
}

pub type PageTableEntry32 = PageTableEntryPrimitive<u32, 34, 2, 10>;
pub type PageTableEntry39 = PageTableEntryPrimitive<u64, 56, 3, 9>;
pub type PageTableEntry48 = PageTableEntryPrimitive<u64, 56, 4, 9>;
pub type PageTableEntry57 = PageTableEntryPrimitive<u64, 56, 5, 9>;
pub type PageEntryImpl = PageTableEntry39;

impl<'a, E: PageTableEntry + 'static> IntoIterator for &'a PageTable<E> {
    type Item = (PageNumber, PageNumber, usize, FlagSet<PageFlag>);

    type IntoIter = PageTableIter<'a, E>;

    fn into_iter(self) -> Self::IntoIter {
        Self::IntoIter {
            root: self,
            level: E::DEPTH - 1,
            inner: None,
            base: 0,
            current: 0,
        }
    }
}

pub struct PageTableIter<'a, E: PageTableEntry + 'static> {
    root: &'a PageTable<E>,
    level: usize,
    inner: Option<Box<UnsafeCell<PageTableIter<'a, E>>>>,
    base: usize,
    current: usize,
}

impl<'a, E: PageTableEntry + 'static> Iterator for PageTableIter<'a, E> {
    type Item = (PageNumber, PageNumber, usize, FlagSet<PageFlag>);

    fn next(&mut self) -> Option<Self::Item> {
        if let Some(inner) = &mut self.inner {
            let res = inner.get_mut().next();
            if res.is_none() {
                self.inner = None;
            }
            res
        } else {
            while self.current < self.root.entries.len() {
                let index = self.current;
                self.current += 1;
                let entry = &self.root.entries[index];
                if entry.is_valid() {
                    let addr =
                        self.base + (index * PageTable::<E>::entry_count().pow(self.level as u32));
                    if entry.is_leaf() {
                        return Some((
                            addr,
                            entry.physical_page_number(),
                            PageTable::<E>::entry_count().pow(self.level as u32),
                            entry.flags(),
                        ));
                    } else {
                        let mut iter = self.root.get_table(index).unwrap().into_iter();
                        iter.base = addr;
                        iter.level = self.level - 1;
                        self.inner = Some(Box::new(UnsafeCell::new(iter)));
                        return self.next();
                    }
                }
            }
            None
        }
    }
}
