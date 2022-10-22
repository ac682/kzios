use erhino_shared::{page::PageLevel, Address, PageNumber};
use flagset::{flags, FlagSet};

#[repr(C)]
pub struct PageTable([PageTableEntry; 512]);

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

impl PageTable {
    pub fn new<'a>(root: PageNumber) -> &'a mut Self {
        unsafe { &mut *((root << 12) as *mut PageTable) }
    }

    pub fn location(&self) -> PageNumber {
        self as *const PageTable as PageNumber
    }

    pub fn entry(&self, index: usize) -> Option<&PageTableEntry> {
        if index >= 512 {
            None
        } else {
            Some(&self.0[index])
        }
    }

    pub fn entry_mut(&mut self, index: usize) -> Option<&mut PageTableEntry> {
        if index >= 512 {
            None
        } else {
            Some(&mut self.0[index])
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
        let val = (ppn << 10 | ((rsw & 0b11) << 8)) as u64 | flags.into().bits();
        self.write(val);
    }

    pub fn set_as_page_table(&mut self, table_root: PageNumber) -> &mut PageTable {
        self.set(table_root, 0, PageTableEntryFlag::Valid);
        PageTable::new(table_root)
    }

    pub fn as_page_table(&mut self) -> &mut PageTable {
        PageTable::new(self.physical_page_number())
    }

    pub fn physical_page_number(&self) -> PageNumber {
        (self.read() >> 10 & 0xFFFFFFFFFFF) as usize
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
