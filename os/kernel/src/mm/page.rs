// Sv39 only

use core::{cell::UnsafeCell, fmt::Debug, mem::size_of, ops::Not};

use alloc::boxed::Box;
use erhino_shared::mem::{Address, PageNumber};
use flagset::{flags, FlagSet};
use hashbrown::HashMap;

use super::frame::FrameTracker;

pub const PAGE_SIZE: usize = 4096;
pub const PAGE_BITS: usize = 12;

pub type PageEntryImpl = PageTableEntry39;

flags! {
    #[repr(u64)]
    pub enum PageEntryFlag: u64{
        Valid = 0b1,
        Readable = 0b10,
        Writeable = 0b100,
        Executable = 0b1000,
        User = 0b1_0000,
        Global = 0b10_0000,
        Accessed = 0b100_0000,
        Dirty = 0b1000_0000,
        Cow = 0b100000000,
        CowWriteable = 0b1000000000,

        PrefabKernelDevice = (PageEntryFlag::Valid | PageEntryFlag::Readable | PageEntryFlag::Writeable | PageEntryFlag::Accessed | PageEntryFlag::Dirty).bits(),
        PrefabKernelProgram = (PageEntryFlag::Valid | PageEntryFlag::Readable | PageEntryFlag::Writeable | PageEntryFlag::Executable | PageEntryFlag::Accessed | PageEntryFlag::Dirty).bits(),
        PrefabKernelTrapframe = (PageEntryFlag::Valid | PageEntryFlag::Readable | PageEntryFlag::Writeable | PageEntryFlag::Accessed | PageEntryFlag::Dirty).bits(),
        PrefabKernelTrampoline = (PageEntryFlag::Valid | PageEntryFlag::Readable | PageEntryFlag::Writeable | PageEntryFlag::Executable | PageEntryFlag::Accessed | PageEntryFlag::Dirty).bits(),
        PrefabUserStack = (PageEntryFlag::Valid | PageEntryFlag::Readable | PageEntryFlag::Writeable | PageEntryFlag::User).bits(),
        PrefabUserTrapframe = (PageEntryFlag::Valid | PageEntryFlag::Readable | PageEntryFlag::Writeable | PageEntryFlag::Accessed | PageEntryFlag::Dirty).bits(),
        PrefabUserTrampoline = (PageEntryFlag::Valid | PageEntryFlag::Readable | PageEntryFlag::Writeable | PageEntryFlag::Executable | PageEntryFlag::Accessed | PageEntryFlag::Dirty).bits(),
    }
}

pub enum PageEntryWriteError {
    LeafExists(PageNumber, FlagSet<PageEntryFlag>),
    BranchExists,
    TrackerUnavailable,
}

pub enum PageEntryType<'table, E: PageTableEntry + 'static> {
    Invalid,
    Leaf(PageNumber, FlagSet<PageEntryFlag>),
    Branch(&'table PageTable<E>),
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
            core::slice::from_raw_parts_mut(
                (location << PAGE_BITS) as *mut E,
                PAGE_SIZE / size_of::<E>(),
            )
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

    pub fn free_entry(&mut self, index: usize) -> Option<PageNumber> {
        let entry = self.entry_mut(index);
        if entry.is_valid() {
            let number = entry.physical_page_number();
            entry.clear();
            if entry.is_leaf() {
                self.managed.remove(&index);
            } else {
                self.branches.remove(&index);
            }
            Some(number)
        } else {
            None
        }
    }

    pub fn is_table_created(&self, index: usize) -> bool {
        let entry = self.entry(index);
        entry.is_valid() && !entry.is_leaf()
    }

    pub fn get_entry_type(&self, index: usize) -> PageEntryType<E> {
        let entry = self.entry(index);
        if entry.is_valid() {
            if entry.is_leaf() {
                PageEntryType::<E>::Leaf(entry.physical_page_number(), entry.flags())
            } else {
                PageEntryType::<E>::Branch(
                    self.get_table(index)
                        .expect("page table management has something went wrong"),
                )
            }
        } else {
            PageEntryType::<E>::Invalid
        }
    }

    pub fn create_leaf<F: Into<FlagSet<PageEntryFlag>>>(
        &mut self,
        index: usize,
        ppn: PageNumber,
        flags: F,
    ) -> Result<(), PageEntryWriteError> {
        let entry = self.entry_mut(index);
        if !entry.is_valid() {
            entry.set(ppn, flags);
            Ok(())
        } else {
            Err(if entry.is_leaf() {
                PageEntryWriteError::LeafExists(entry.physical_page_number(), entry.flags())
            } else {
                PageEntryWriteError::BranchExists
            })
        }
    }

    pub fn ensure_leaf_created<F: Into<FlagSet<PageEntryFlag>>>(
        &mut self,
        index: usize,
        ppn: PageNumber,
        flags: F,
    ) -> Result<bool, PageEntryWriteError> {
        let entry = self.entry_mut(index);
        if entry.is_valid() {
            if entry.is_leaf() {
                if entry.physical_page_number() == ppn {
                    entry.set_flags(
                        flags.into()
                            & (PageEntryFlag::Readable
                                | PageEntryFlag::Writeable
                                | PageEntryFlag::Executable),
                    );
                    Ok(false)
                } else {
                    Err(PageEntryWriteError::LeafExists(
                        entry.physical_page_number(),
                        entry.flags(),
                    ))
                }
            } else {
                Err(PageEntryWriteError::BranchExists)
            }
        } else {
            entry.set(ppn, flags);
            Ok(true)
        }
    }

    pub fn create_managed_leaf<F: Into<FlagSet<PageEntryFlag>>>(
        &mut self,
        index: usize,
        tracker: FrameTracker,
        flags: F,
    ) -> Result<(), PageEntryWriteError> {
        let entry = self.entry_mut(index);
        let ppn = tracker.start();
        if !entry.is_valid() {
            entry.set(ppn, flags);
            self.managed.insert(index, tracker);
            Ok(())
        } else {
            Err(if entry.is_leaf() {
                PageEntryWriteError::LeafExists(entry.physical_page_number(), entry.flags())
            } else {
                PageEntryWriteError::BranchExists
            })
        }
    }

    pub fn ensure_managed_leaf_created<
        F: Into<FlagSet<PageEntryFlag>>,
        Factory: FnOnce() -> Option<FrameTracker>,
    >(
        &mut self,
        index: usize,
        tracker_factory: Factory,
        flags: F,
    ) -> Result<bool, PageEntryWriteError> {
        let entry = self.entry_mut(index);
        if entry.is_valid() {
            if entry.is_leaf() {
                entry.set_flags(
                    flags.into()
                        & (PageEntryFlag::Readable
                            | PageEntryFlag::Writeable
                            | PageEntryFlag::Executable),
                );
                Ok(false)
            } else {
                Err(PageEntryWriteError::BranchExists)
            }
        } else {
            if let Some(tracker) = tracker_factory() {
                self.create_managed_leaf(index, tracker, flags)
                    .map(|_| true)
            } else {
                Err(PageEntryWriteError::TrackerUnavailable)
            }
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

    pub fn get_table_mut(&mut self, index: usize) -> Option<&mut PageTable<E>> {
        let entry = self.entry(index);
        if entry.is_valid() && !entry.is_leaf() {
            self.branches.get_mut(&index)
        } else {
            None
        }
    }

    pub fn ensure_table_created<F: FnOnce() -> Option<FrameTracker>>(
        &mut self,
        index: usize,
        frame_factory: F,
    ) -> Result<&mut PageTable<E>, PageEntryWriteError> {
        let entry = self.entry_mut(index);

        if entry.is_valid() {
            if !entry.is_leaf() {
                Ok(self
                    .branches
                    .get_mut(&index)
                    .expect("page table management went wrong"))
            } else {
                Err(PageEntryWriteError::LeafExists(
                    entry.physical_page_number(),
                    entry.flags(),
                ))
            }
        } else {
            if let Some(frame) = frame_factory() {
                let ppn = frame.start();
                entry.set(ppn, PageEntryFlag::Valid);
                let table = PageTable::<E>::from(ppn);
                self.managed.insert(index, frame);
                self.branches.insert(index, table);
                Ok(self
                    .branches
                    .get_mut(&index)
                    .expect("page table management went wrong"))
            } else {
                Err(PageEntryWriteError::TrackerUnavailable)
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
    fn has_flags<F: Into<FlagSet<PageEntryFlag>>>(&self, flags: F) -> bool;
    fn set_flags<F: Into<FlagSet<PageEntryFlag>>>(&mut self, flags: F);
    fn clear_flags<F: Into<FlagSet<PageEntryFlag>>>(&mut self, flags: F);
    fn flags(&self) -> FlagSet<PageEntryFlag>;
    fn physical_page_number(&self) -> PageNumber;
    fn set<F: Into<FlagSet<PageEntryFlag>>>(&mut self, ppn: PageNumber, flags: F);
    fn clear(&mut self);
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
        1usize << (DEPTH * SIZE + PAGE_BITS - 1)
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

    fn set_flags<F: Into<FlagSet<PageEntryFlag>>>(&mut self, flags: F) {
        let pre = self.0.into() | flags.into().bits();
        if let Ok(p) = P::try_from(pre) {
            self.0 = p
        }
    }

    fn clear_flags<F: Into<FlagSet<PageEntryFlag>>>(&mut self, flags: F) {
        let pre = self.0.into() & (!0u64 - flags.into().bits());
        if let Ok(p) = P::try_from(pre) {
            self.0 = p
        }
    }

    fn has_flags<F: Into<FlagSet<PageEntryFlag>>>(&self, flags: F) -> bool {
        self.0.into() & flags.into().bits() != 0
    }

    fn flags(&self) -> FlagSet<PageEntryFlag> {
        FlagSet::new(self.0.into() & FlagSet::<PageEntryFlag>::full().bits()).unwrap()
    }

    fn physical_page_number(&self) -> PageNumber {
        ((self.0.into() & ((1 << Self::LENGTH) - 1)) >> 10) as PageNumber
    }

    fn set<F: Into<FlagSet<PageEntryFlag>>>(&mut self, ppn: PageNumber, flags: F) {
        let pre = ((ppn as u64) << 10) + flags.into().bits();
        if let Ok(p) = P::try_from(pre) {
            self.0 = p
        }
    }

    fn clear(&mut self) {
        if let Ok(p) = P::try_from(0) {
            self.0 = p;
        }
    }
}

#[allow(unused)]
pub type PageTableEntry32 = PageTableEntryPrimitive<u32, 34, 2, 10>;
#[allow(unused)]
pub type PageTableEntry39 = PageTableEntryPrimitive<u64, 56, 3, 9>;
#[allow(unused)]
pub type PageTableEntry48 = PageTableEntryPrimitive<u64, 56, 4, 9>;
#[allow(unused)]
pub type PageTableEntry57 = PageTableEntryPrimitive<u64, 56, 5, 9>;

impl<'a, E: PageTableEntry + 'static> IntoIterator for &'a PageTable<E> {
    type Item = (PageNumber, PageNumber, usize, FlagSet<PageEntryFlag>);

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
    type Item = (PageNumber, PageNumber, usize, FlagSet<PageEntryFlag>);

    fn next(&mut self) -> Option<Self::Item> {
        if let Some(inner) = &mut self.inner {
            let res = inner.get_mut().next();
            if res.is_none() {
                self.inner = None;
                self.next()
            } else {
                res
            }
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
