use core::ops::BitOr;

use flagset::{flags, FlagSet};

use crate::paged::address::{PhysicalAddress, VirtualAddress};
use crate::paged::free;
use crate::{alloc, println};

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

    pub fn ensure_created(
        &self,
        vpn: u64,
        flags: impl Into<FlagSet<PageTableEntryFlag>>,
    ) -> Option<u64> {
        let flags_into = flags.into();
        if let Ok(entry) = self.locate(vpn) {
            return if entry.is_valid() && entry.is_leaf() {
                if entry.flags().bits() != flags_into.bits() {
                    entry.write_bitor(flags_into.bits());
                }
                Some(entry.physical_page_number())
            } else {
                if let Some(ppn) = alloc() {
                    entry.set(ppn, 0, flags_into);
                    Some(ppn)
                } else {
                    None
                }
            };
        } else {
            None
        }
    }

    pub fn map(
        &self,
        ppn: u64,
        vpn: u64,
        flags: impl Into<FlagSet<PageTableEntryFlag>>,
    ) -> Result<(), ()> {
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

    pub fn unmap(&self, vpn: u64) -> Result<u64, ()> {
        let entry = self.locate(vpn)?;
        if entry.is_valid() && entry.is_leaf() {
            let ppn = entry.physical_page_number();
            entry.clear();
            Ok(ppn)
        } else {
            Err(())
        }
    }

    pub fn page_number(&self) -> u64 {
        self.page_number
    }

    pub fn level(&self) -> usize {
        self.level
    }

    pub fn free(self) {
        for i in 0..512 {
            let entry = self.entry(i);
            if entry.is_valid() {
                if entry.is_leaf() {
                    free(entry.physical_page_number());
                    entry.clear();
                } else {
                    entry.as_page_table(self.level - 1).free();
                }
            }
        }
        free(self.page_number);
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
        self.read() & 0b1 != 0
    }

    pub fn is_leaf(&self) -> bool {
        self.read() & 0b1110 != 0
    }

    pub fn read(&self) -> u64 {
        unsafe {
            let reg = self.address as *const u64;
            reg.read_volatile() as u64
        }
    }

    pub fn write(&self, bits: u64) {
        unsafe {
            let reg = self.address as *mut u64;
            reg.write_volatile(bits);
        }
    }

    pub fn write_bitor(&self, bits: u64) {
        unsafe {
            let reg = self.address as *mut u64;
            let old = reg.read_volatile();
            reg.write_volatile(bits | old);
        }
    }

    pub fn set(&self, ppn: u64, rsw: u64, flags: impl Into<FlagSet<PageTableEntryFlag>>) {
        unsafe {
            let flag_bits = flags.into().bits();
            let reg = self.address as *mut u64;
            let bits = ((ppn << 10) | (rsw << 8)) | flag_bits;
            reg.write_volatile(bits);
        }
    }

    pub fn clear(&self) {
        unsafe {
            let reg = self.address as *mut u8;
            reg.write_volatile(0);
        }
    }

    pub fn physical_page_number(&self) -> u64 {
        self.read() >> 10 & 0xFFFFFFFFFFF
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

    pub fn as_page_table(&self, level: usize) -> PageTable {
        let ppn = (self.read() >> 10) & 0x1FFFFFFFFFFF;
        PageTable::new(level, ppn)
    }

    pub fn set_as_page_table(&self, ppn: u64, level: usize) -> PageTable {
        self.set(ppn, 0, PageTableEntryFlag::Valid);
        self.as_page_table(level)
    }

    pub fn flags(&self) -> FlagSet<PageTableEntryFlag> {
        FlagSet::new(self.read() & 0b1111_1111).unwrap()
    }
}
