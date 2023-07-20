// Sv39 only

use core::{
    fmt::Debug,
    mem::size_of,
    ops::{BitAnd, BitOr},
};

use alloc::vec::Vec;
use erhino_shared::mem::PageNumber;
use flagset::{flags, FlagSet};
use hashbrown::HashMap;

use super::frame::{self, FrameTracker};

const PAGE_SIZE: usize = 4096;

pub enum PageTableError {
    EntryNotFound,
    EntryDefinitionConflicts,
    EntryUndefined,
    EntryNotLeaf,
    EntryNotBranch,
    WrongLevel,
    PhysicalPageNumberUnaligned,
}

flags! {
    pub enum PageTableEntryFlag: u64{
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

        UserReadWrite = (PageTableEntryFlag::User | PageTableEntryFlag::Readable | PageTableEntryFlag::Writeable | PageTableEntryFlag::Valid).bits(),
    }
}

pub struct PageTable<E: PageTableEntry + Sized + 'static> {
    location: FrameTracker,
    entries: &'static mut [E],
    subs: Vec<PageTable<E>>,
    managed: HashMap<PageNumber, FrameTracker>,
}

impl<E: PageTableEntry + Sized + 'static> PageTable<E> {
    pub fn from(tracker: FrameTracker) -> Self {
        let entries = unsafe {
            core::slice::from_raw_parts_mut(
                (tracker.start() << 12) as *mut E,
                PAGE_SIZE / size_of::<E>(),
            )
            .try_into()
            .unwrap()
        };
        Self {
            location: tracker,
            entries: entries,
            subs: Vec::new(),
            managed: HashMap::new(),
        }
    }

    pub fn page_number(&self) -> PageNumber {
        self.location.start()
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

    // check is_entry_crated before invocation
    pub fn create_entry<F: Into<FlagSet<PageTableEntryFlag>>>(
        &mut self,
        index: usize,
        tracker: FrameTracker,
        flags: F,
    ) {
        let number = tracker.start();
        let entry = self.entry_mut(index);
        entry.set(number, flags);
        self.managed.insert(number, tracker);
    }

    // // check is_entry_crated before invocation, too
    pub fn delete_entry(&mut self, index: usize) {
        let entry = self.entry_mut(index);
        let number = entry.physical_page_number();
        entry.clear_flag(PageTableEntryFlag::Valid);
        self.managed.remove(&number);
    }

    pub fn get_entry(&mut self, index: usize) -> Option<&mut E> {
        let entry = self.entry_mut(index);
        if entry.is_valid() {
            Some(entry)
        } else {
            None
        }
    }

    pub const fn entry_count() -> usize {
        PAGE_SIZE / size_of::<E>()
    }
}

pub trait PageTableEntry {
    const LENGTH: usize;
    const DEPTH: usize;
    const SIZE: usize;
    fn is_leaf(&self) -> bool;
    fn is_valid(&self) -> bool;
    fn has_flag(&self, flag: PageTableEntryFlag) -> bool;
    fn set_flag(&mut self, flag: PageTableEntryFlag);
    fn clear_flag(&mut self, flag: PageTableEntryFlag);
    fn flags(&self) -> FlagSet<PageTableEntryFlag>;
    fn physical_page_number(&self) -> PageNumber;
    fn set<F: Into<FlagSet<PageTableEntryFlag>>>(&mut self, ppn: PageNumber, flags: F);
}

pub struct PageTableEntryPrimitive<
    P: Clone + Copy + Into<u64> + TryFrom<u64>,
    const LENGTH: usize,
    const DEPTH: usize,
    const SIZE: usize,
>(P);

impl<
        P: Clone + Copy + Into<u64> + TryFrom<u64>,
        const LENGTH: usize,
        const DEPTH: usize,
        const SIZE: usize,
    > PageTableEntry for PageTableEntryPrimitive<P, LENGTH, DEPTH, SIZE>
{
    const LENGTH: usize = LENGTH;

    const DEPTH: usize = DEPTH;

    const SIZE: usize = SIZE;
    fn is_leaf(&self) -> bool {
        self.0.into() & 0b1110 != 0
    }

    fn is_valid(&self) -> bool {
        self.0.into() & 0b1 != 0
    }

    fn set_flag(&mut self, flag: PageTableEntryFlag) {
        let pre = self.0.into() | flag as u64;
        if let Ok(p) = P::try_from(pre) {
            self.0 = p
        }
    }

    fn clear_flag(&mut self, flag: PageTableEntryFlag) {
        let pre = self.0.into() & (!0u64 - flag as u64);
        if let Ok(p) = P::try_from(pre) {
            self.0 = p
        }
    }

    fn has_flag(&self, flag: PageTableEntryFlag) -> bool {
        self.0.into() & flag as u64 != 0
    }

    fn flags(&self) -> FlagSet<PageTableEntryFlag> {
        FlagSet::new(self.0.into() & FlagSet::<PageTableEntryFlag>::full().bits()).unwrap()
    }

    fn physical_page_number(&self) -> PageNumber {
        ((self.0.into() & (1 << Self::LENGTH - 1)) >> 10) as PageNumber
    }

    fn set<F: Into<FlagSet<PageTableEntryFlag>>>(&mut self, ppn: PageNumber, flags: F) {
        let pre = (ppn as u64) << 10 + flags.into().bits();
        if let Ok(p) = P::try_from(pre) {
            self.0 = p
        }
    }
}

pub type PageTableEntry32 = PageTableEntryPrimitive<u32, 34, 2, 10>;
pub type PageTableEntry39 = PageTableEntryPrimitive<u64, 56, 3, 9>;
pub type PageTableEntry48 = PageTableEntryPrimitive<u64, 56, 4, 9>;
pub type PageTableEntry57 = PageTableEntryPrimitive<u64, 56, 5, 9>;
