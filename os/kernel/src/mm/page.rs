// Sv39 only

use core::{
    mem::size_of,
    ops::{BitAnd, BitOr},
};

use alloc::vec::Vec;
use erhino_shared::mem::PageNumber;
use flagset::{flags, FlagSet};

use super::{
    frame::{self, FrameTracker},
    unit::MemoryUnit,
};

const PAGE_SIZE: usize = 4096;

const fn entry_count<E: Sized>() -> usize {
    PAGE_SIZE / size_of::<E>()
}

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

        UserReadWrite = (PageTableEntryFlag::User | PageTableEntryFlag::Readable | PageTableEntryFlag::Writeable | PageTableEntryFlag::Valid).bits(),
    }
}

pub struct PageTable<E: PageTableEntry + Sized + 'static> {
    location: PageNumber,
    entries: &'static mut [E],
    managed: Vec<FrameTracker>,
}

impl<E: PageTableEntry + Sized> PageTable<E> {
    pub fn new() -> Option<Self> {
        if let Some(tracker) = frame::borrow(1) {
            if let Ok(entries) = unsafe {
                core::slice::from_raw_parts_mut(
                    (tracker.start() << 12) as *mut E,
                    entry_count::<E>(),
                )
                .try_into()
            } {
                Some(Self {
                    location: tracker.start(),
                    entries: entries,
                    managed: alloc::vec![tracker],
                })
            } else {
                None
            }
        } else {
            None
        }
    }

    pub fn from(location: PageNumber) -> Option<Self> {
        if let Ok(entries) = unsafe {
            core::slice::from_raw_parts_mut((location << 12) as *mut E, entry_count::<E>())
                .try_into()
        } {
            Some(Self {
                location: location,
                entries: entries,
                managed: Vec::new(),
            })
        } else {
            None
        }
    }

    pub fn page_number(&self) -> PageNumber {
        self.location
    }

    pub fn entry(&self, index: usize) -> Option<&E> {
        if index >= entry_count::<E>() {
            None
        } else {
            Some(&self.entries[index])
        }
    }

    pub fn entry_mut(&mut self, index: usize) -> Option<&mut E> {
        if index >= entry_count::<E>() {
            None
        } else {
            Some(&mut self.entries[index])
        }
    }
}

pub trait PageTableEntry {
    const MODE: usize;
    const DEPTH: usize;
    const SEGMENT: usize;
    fn is_leaf(&self) -> bool;
    fn is_valid(&self) -> bool;
    fn is_readable(&self) -> bool;
    fn is_writeable(&self) -> bool;
    fn is_executable(&self) -> bool;
    fn flags(&self) -> FlagSet<PageTableEntryFlag>;
    fn physical_page_number(&self) -> PageNumber;
    fn is_cow(&self) -> bool;
    fn is_cow_and_writeable(&self) -> bool;
}

pub struct PageTableEntryPrimitive<
    P: Clone + Copy + BitAnd<P> + BitOr<P>,
    const MODE: usize,
    const DEPTH: usize,
    const SEGMENT: usize,
>(P);

impl<
        P: Clone + Copy + BitAnd<P> + BitOr<P> + Into<u64>,
        const MODE: usize,
        const DEPTH: usize,
        const SEGMENT: usize,
    > PageTableEntry for PageTableEntryPrimitive<P, MODE, DEPTH, SEGMENT>
{
    const MODE: usize = MODE;

    const DEPTH: usize = DEPTH;

    const SEGMENT: usize = SEGMENT;
    fn is_leaf(&self) -> bool {
        self.0 & 0b1110 != 0
    }

    fn is_valid(&self) -> bool {
        self.0 & 0b1 != 0
    }

    fn is_readable(&self) -> bool {
        self.0 & 0b10 != 0
    }

    fn is_writeable(&self) -> bool {
        self.0 & 0b100 != 0
    }

    fn is_executable(&self) -> bool {
        self.0 & 0b1000 != 0
    }

    fn flags(&self) -> FlagSet<PageTableEntryFlag> {
        FlagSet::new(self.0.into() & FlagSet::<PageTableEntryFlag>::full().bits()).unwrap()
    }

    fn physical_page_number(&self) -> PageNumber {
        ((self.0 & ((1 << MODE - 1) - 0b11_1111_1111)) >> 10) as PageNumber
    }

    fn is_cow(&self) -> bool {
        self.0 & (0b11 << 8) > 0
    }

    fn is_cow_and_writeable(&self) -> bool {
        self.0 & (0b11 << 8) == 0b11_0000_0000
    }
}

pub type PageTableEntry32 = PageTableEntryPrimitive<u32, 32, 2, 10>;
pub type PageTableEntry39 = PageTableEntryPrimitive<u64, 39, 3, 9>;
pub type PageTableEntry48 = PageTableEntryPrimitive<u64, 48, 4, 9>;
pub type PageTableEntry57 = PageTableEntryPrimitive<u64, 57, 5, 9>;
