use crate::paged::address::{PhysicalAddress, VirtualAddress};
use crate::primitive::mmio::mmio_read;
use crate::{alloc, println};

pub struct PageTable {
    page_number: usize,
    level: usize,
}

impl PageTable {
    pub fn new(level: usize, location: usize) -> Self {
        Self {
            level,
            page_number: location,
        }
    }

    pub fn entry(&self, index: usize) -> PageTableEntry {
        let address = (self.page_number << 12) + (index * 8);
        PageTableEntry::new(PhysicalAddress::from(address))
    }

    pub fn map(&self, ppn: usize, vpn: usize, flags: usize) -> Result<(), ()> {
        if self.level != 0 {
            let index = (vpn >> (9 * self.level)) & 0x1ff;
            let entry = self.entry(index);
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
                table.map(ppn, vpn, flags)
            } else {
                Err(())
            }
        } else {
            self.entry(vpn & 0x1ff).set(ppn, 0, flags);
            Ok(())
        }
    }

    pub fn page_number(&self) -> usize {
        self.page_number
    }

    pub fn level(&self) -> usize {
        self.level
    }
}

pub struct PageTableEntry {
    address: usize,
}

impl PageTableEntry {
    pub fn new(address: PhysicalAddress) -> Self {
        Self {
            address: usize::from(address),
        }
    }

    pub fn is_valid(&self) -> bool {
        self.val() & 0b1 != 0
    }

    pub fn is_leaf(&self) -> bool {
        self.val() & 0b1110 != 0
    }

    pub fn val(&self) -> usize {
        unsafe {
            let reg = self.address as *mut usize;
            reg.read_volatile()
        }
    }

    pub fn set(&self, ppn: usize, rsw: usize, flags: usize) {
        unsafe {
            let reg = self.address as *mut usize;
            reg.write_volatile((ppn << 10) | (rsw << 8) | flags);
        }
    }

    pub fn physical_page_number(&self) -> usize {
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

    pub fn set_as_page_table(&self, ppn: usize, level: usize) -> PageTable {
        self.set(ppn, 0, 0b0001);
        self.as_page_table(level)
    }
}
