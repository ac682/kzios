use erhino_shared::mem::{page::PageLevel, Address, PageNumber};
use flagset::{flags, FlagSet};

use crate::println;

#[derive(Debug)]
pub struct PageTable {
    entries: [PageTableEntry; 512],
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

impl PageTable {
    pub fn new<'a>(root: PageNumber) -> &'a mut Self {
        let mut res = unsafe { ((root << 12) as *mut PageTable).as_mut().unwrap() };
        for i in 0..512usize {
            res.entries[i].write(0);
        }
        res
    }

    pub fn from_exist<'a>(root: PageNumber) -> &'a mut Self {
        unsafe { &mut *((root << 12) as *mut PageTable) }
    }

    pub fn location(&self) -> PageNumber {
        self as *const PageTable as PageNumber >> 12
    }

    pub fn entry(&self, index: usize) -> Option<&PageTableEntry> {
        if index >= 512 {
            None
        } else {
            Some(&self.entries[index])
        }
    }

    pub fn entry_mut(&mut self, index: usize) -> Option<&mut PageTableEntry> {
        if index >= 512 {
            None
        } else {
            Some(&mut self.entries[index])
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

#[derive(Debug)]
pub struct PageTableEntry(u64);

impl PageTableEntry {
    pub fn read(&self) -> u64 {
        self.0
    }

    pub fn write(&mut self, val: u64) {
        self.0 = val;
    }

    pub fn write_bitor(&mut self, val: u64) {
        self.0 = self.0 | val
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

    pub fn set_cow(&mut self) {
        let cow = 0b1 | if self.is_writeable() { 0b10 } else { 0b00 };
        self.0 = (self.0 & !0b100) | (cow << 8);
    }

    pub fn is_cow(&self) -> bool{
        self.0 & (0b11 << 8) > 0
    }

    pub fn is_cow_and_writeable(&self) -> bool{
        self.0 & (0b11 << 8) == 0b11_0000_0000
    }

    pub fn set_as_page_table_mut(&mut self, table_root: PageNumber) -> &mut PageTable {
        self.set(table_root, 0, PageTableEntryFlag::Valid);
        PageTable::new(table_root)
    }

    pub fn as_page_table(&self) -> &PageTable {
        PageTable::from_exist(self.physical_page_number())
    }

    pub fn as_page_table_mut(&mut self) -> &mut PageTable {
        PageTable::from_exist(self.physical_page_number())
    }

    pub fn physical_page_number(&self) -> PageNumber {
        (self.0 >> 10 & 0xFFFFFFFFFFF) as PageNumber
    }

    pub fn is_valid(&self) -> bool {
        self.0 & 0b1 != 0
    }

    pub fn is_leaf(&self) -> bool {
        self.0 & 0b1110 != 0
    }

    pub fn is_readable(&self) -> bool {
        self.0 >> 1 & 1 != 0
    }

    pub fn is_writeable(&self) -> bool {
        self.0 >> 2 & 1 != 0
    }

    pub fn is_executable(&self) -> bool {
        self.0 >> 3 & 1 != 0
    }

    pub fn flags(&self) -> FlagSet<PageTableEntryFlag> {
        FlagSet::new(self.0 & FlagSet::<PageTableEntryFlag>::full().bits()).unwrap()
    }
}

impl<'root> IntoIterator for &'root PageTable {
    type Item = &'root PageTableEntry;

    type IntoIter = PageTableIter<'root>;

    fn into_iter(self) -> Self::IntoIter {
        Self::IntoIter {
            root: self,
            current: 0,
        }
    }
}

pub struct PageTableIter<'root> {
    root: &'root PageTable,
    current: usize,
}

impl<'root> Iterator for PageTableIter<'root> {
    type Item = &'root PageTableEntry;

    fn next(&mut self) -> Option<Self::Item> {
        while self.current < self.root.entries.len() {
            let index = self.current;
            self.current += 1;
            if self.root.entries[index].is_valid() {
                return Some(&self.root.entries[index]);
            }
        }
        None
    }
}
