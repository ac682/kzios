use erhino_shared::{Address, PageNumber};
use flagset::{flags, FlagSet};

pub struct PageTable<'pt> {
    root: PageNumber,
    level: PageLevel,
    held_entries: &'pt mut [PageTableEntry; 512],
}

#[derive(Clone, Copy, PartialEq)]
pub enum PageLevel {
    Kilo,
    Mega,
    Giga,
}

#[derive(Debug)]
pub enum PageTableError {
    EntryNotFound,
    EntryDefinitionConflicts,
    EntryUndefined,
    EntryNotLeaf,
    EntryNotBranch,
    WrongLevel,
    PhysicalPageNumberUnaligned,
}

impl PageLevel {
    pub fn value(&self) -> u8 {
        match self {
            Self::Kilo => 0,
            Self::Mega => 1,
            Self::Giga => 2,
        }
    }

    pub fn size(&self) -> usize {
        match self {
            Self::Kilo => 8 * 512,
            Self::Mega => 8 * 512 * 512,
            Self::Giga => 8 * 512 * 512 * 512,
        }
    }

    pub fn next_level(&self) -> Option<PageLevel> {
        match self {
            PageLevel::Giga => Some(PageLevel::Mega),
            PageLevel::Mega => Some(PageLevel::Kilo),
            PageLevel::Kilo => None,
        }
    }

    pub fn floor(&self, page_number: PageNumber) -> PageNumber {
        page_number >> (9 * self.value()) << (9 * self.value())
    }

    pub fn ceil(&self, page_number: PageNumber) -> PageNumber {
        (page_number >> (9 * self.value()) << (9 * self.value())) + self.size() as PageNumber
    }

    pub fn measure(&self, page_count: usize) -> usize {
        page_count >> (9 * self.value())
    }

    pub fn extract(&self, page_number: PageNumber) -> usize {
        (page_number >> (9 * self.value())) as usize
    }
}

impl<'pt> PageTable<'pt> {
    pub fn new(root: PageNumber, level: PageLevel) -> Self {
        Self {
            root,
            level,
            held_entries: unsafe { &mut *((root << 12) as *mut [PageTableEntry; 512]) },
        }
    }

    pub fn is_page_number_aligned(page_number: PageNumber, level: PageLevel) -> bool {
        let n = level.value() as u32 * 9; // ranging in [0, 9, 18]
        let mask = 2u64.pow(n) - 1;
        page_number & mask == 0
    }

    pub fn location(&self) -> PageNumber {
        self.root
    }

    pub fn level(&self) -> PageLevel {
        self.level
    }

    pub fn entry(&'pt self, index: usize) -> Option<&'pt PageTableEntry> {
        if index >= 512 {
            None
        } else {
            Some(&self.held_entries[index])
        }
    }

    pub fn entry_mut(&'pt mut self, index: usize) -> Option<&mut PageTableEntry> {
        if index >= 512 {
            None
        } else {
            Some(&mut self.held_entries[index])
        }
    }
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

#[repr(C)]
pub struct PageTableEntry(u64);

impl PageTableEntry {
    pub fn read(&self) -> u64 {
        self.0
    }

    pub fn write(&mut self, val: u64) {
        self.0 = val;
    }

    pub fn write_bitor(&mut self, val: u64) {
        self.write(self.read() | val)
    }

    pub fn set<F: Into<FlagSet<PageTableEntryFlag>>>(
        &mut self,
        ppn: PageNumber,
        rsw: usize,
        flags: F,
    ) {
        let val = ppn << 10 | ((rsw & 0b11) << 8) as u64 | flags.into().bits();
        self.write(val);
    }

    pub fn set_as_page_table(&mut self, table_root: PageNumber, level: PageLevel) -> PageTable{
        self.set(table_root, 0, PageTableEntryFlag::Valid);
        PageTable::new(table_root, level)
    }

    pub fn as_page_table(&self, level: PageLevel) -> PageTable{
        PageTable::new(self.physical_page_number(), level)
    }

    pub fn physical_page_number(&self) -> PageNumber {
        self.read() >> 10 & 0xFFFFFFFFFFF
    }

    pub fn is_valid(&self) -> bool {
        self.read() & 0b1 != 0
    }

    pub fn is_leaf(&self) -> bool {
        self.read() & 0b1110 != 0
    }

    pub fn is_readable(&self) -> bool {
        self.read() >> 1 & 1 != 0
    }

    pub fn is_writeable(&self) -> bool {
        self.read() >> 2 & 1 != 0
    }

    pub fn is_executable(&self) -> bool {
        self.read() >> 3 & 1 != 0
    }

    pub fn flags(&self) -> FlagSet<PageTableEntryFlag> {
        FlagSet::new(self.read() & 0b1111_1111).unwrap()
    }
}
