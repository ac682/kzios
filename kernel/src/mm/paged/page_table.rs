use core::ops::BitOr;

use flagset::{flags, FlagSet};

use crate::{alloc, println};
use crate::paged::address::{PhysicalAddress, VirtualAddress};

pub struct PageTable {
    page_number: u64,
    level: usize,
}

impl PageTable {
    pub fn new(level: usize, location: u64) -> Self {
        Self {
            level,
            page_number: location,
        }
    }

    pub fn entry(&self, index: usize) -> PageTableEntry {
        let address = (self.page_number << 12) + (index * 8) as u64;
        PageTableEntry::new(PhysicalAddress::from(address))
    }

    pub fn locate(&self, vpn: u64) -> Result<PageTableEntry, ()> {
        if self.level != 0 {
            let index = (vpn >> (9 * self.level)) & 0x1ff;
            let entry = self.entry(index as usize);
            let table_option = if entry.is_valid() && !entry.is_leaf() {
                Some(entry.as_page_table(self.level - 1))
            } else {
                if let Some(frame) = alloc() {
                    Some(entry.set_as_page_table(frame, self.level - 1))
                } else {
                    None
                }
            };
            if let Some(table) = table_option {
                table.locate(vpn)
            } else {
                Err(())
            }
        } else {
            Ok(self.entry(vpn as usize & 0x1ff))
        }
    }

    pub fn ensure_created(&self, vpn: u64, flags: impl Into<FlagSet<PageTableEntryFlag>>) -> Option<u64> {
        if let Ok(entry) = self.locate(vpn) {
            return if entry.is_valid() && entry.is_leaf() {
                Some(entry.physical_page_number())
            } else {
                let ppn = alloc().unwrap();
                entry.set(ppn, 0, flags);
                Some(ppn)
            };
        }
        None
    }

    pub fn map(&self, ppn: u64, vpn: u64, flags: impl Into<FlagSet<PageTableEntryFlag>>) -> Result<(), ()> {
        let entry = self.locate(vpn)?;
        if entry.is_leaf() && entry.is_valid() {
            // is set do not overwrite
            println!("overwrite");
            Err(())
        } else {
            entry.set(ppn, 0, flags);
            Ok(())
        }
    }

    pub fn page_number(&self) -> u64 {
        self.page_number
    }

    pub fn level(&self) -> usize {
        self.level
    }

    pub fn fork(&self) -> Option<PageTable> {
        if let Some(root_page_number) = alloc() {
            let res = PageTable::new(2, root_page_number);
            self.enumerate(|pte, vpn| {
                res.map(pte.physical_page_number(), vpn, pte.flags());
            });
            Some(res)
        } else {
            None
        }
    }

    pub fn enumerate(&self, func: impl Fn(&PageTableEntry, u64)) {
        let table2 = self;
        for vpn2 in 0..512 {
            let pte2 = table2.entry(vpn2);
            if pte2.is_valid() {
                if pte2.is_leaf() {
                    // G page
                    todo!("invalid page table at {:#x}#{}", table2.page_number(), vpn2);
                } else {
                    let table1 = pte2.as_page_table(1);
                    for vpn1 in 0..512 {
                        let pte1 = table1.entry(vpn1);
                        if pte1.is_valid() {
                            if pte1.is_leaf() {
                                todo!("invalid page table at {:#x}#{}", table2.page_number(), vpn1);
                            } else {
                                let table0 = pte1.as_page_table(0);
                                for vpn0 in 0..512 {
                                    let pte0 = table0.entry(vpn0);
                                    if pte0.is_valid() && pte0.is_leaf() {
                                        func(&pte0, ((vpn2 >> 18) + (vpn1 >> 9) + vpn0) as u64);
                                    }
                                }
                            }
                        }
                    }
                }
            }
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

pub struct PageTableEntry {
    address: u64,
}

impl PageTableEntry {
    pub fn new(address: PhysicalAddress) -> Self {
        Self {
            address: u64::from(address),
        }
    }

    pub fn is_valid(&self) -> bool {
        self.val() & 0b1 != 0
    }

    pub fn is_leaf(&self) -> bool {
        self.val() & 0b1110 != 0
    }

    pub fn val(&self) -> u64 {
        unsafe {
            let reg = self.address as *const u64;
            reg.read_volatile() as u64
        }
    }

    pub fn set(&self, ppn: u64, rsw: u64, flags: impl Into<FlagSet<PageTableEntryFlag>>) {
        unsafe {
            let bits = flags.into().bits();
            let reg = self.address as *mut u64;
            reg.write_volatile(((ppn << 10) | (rsw << 8)) | bits);
        }
    }

    pub fn physical_page_number(&self) -> u64 {
        self.val() >> 10 & 0xFFFFFFFFFFF
    }

    pub fn is_readable(&self) -> bool {
        self.val() >> 1 & 1 != 0
    }

    pub fn is_writeable(&self) -> bool {
        self.val() >> 2 & 1 != 0
    }

    pub fn is_executable(&self) -> bool {
        self.val() >> 3 & 1 != 0
    }

    pub fn as_page_table(&self, level: usize) -> PageTable {
        let ppn = (self.val() >> 10) & 0x1FFFFFFFFFFF;
        PageTable::new(level, ppn)
    }

    pub fn set_as_page_table(&self, ppn: u64, level: usize) -> PageTable {
        self.set(ppn, 0, PageTableEntryFlag::Valid);
        self.as_page_table(level)
    }

    pub fn flags(&self) -> FlagSet<PageTableEntryFlag> {
        FlagSet::new(self.val() & 0b1111_1111).unwrap()
    }
}
