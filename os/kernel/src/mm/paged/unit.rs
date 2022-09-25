use core::slice;

use flagset::FlagSet;
use riscv::{asm::sfence_vma_all, register::satp};

use crate::paged::page_table::PageTableEntryFlag;
use crate::{alloc, println};

use super::page_table::{PageTable, PageTableEntry};

pub struct MemoryUnit {
    root: Option<PageTable>,
}

impl MemoryUnit {
    pub fn new() -> Self {
        Self { root: None }
    }

    pub fn init(&mut self, root: PageTable) {
        self.root = Some(root)
    }

    pub fn map(&self, ppn: u64, vpn: u64, flags: impl Into<FlagSet<PageTableEntryFlag>>) {
        if let Some(table) = &self.root {
            table.map(ppn, vpn, flags.into() | PageTableEntryFlag::Valid);
        }
    }

    pub fn unmap(&self, vpn: u64) {
        if let Some(table) = &self.root {
            table.unmap(vpn);
        }
    }

    pub fn fill(&self, vpn: u64, count: usize, flags: impl Into<FlagSet<PageTableEntryFlag>>) {
        let f = flags.into();
        let cnt = match count {
            0 => 1,
            _ => count,
        };
        if let Some(table) = &self.root {
            for i in 0..cnt {
                table
                    .map(alloc().unwrap(), vpn + i as u64, f)
                    .expect("PANIC!");
            }
        }
    }

    pub fn ensure_created(
        &self,
        vpn: u64,
        flags: impl Into<FlagSet<PageTableEntryFlag>>,
    ) -> Option<u64> {
        if let Some(table) = &self.root {
            table.ensure_created(vpn, flags.into() | PageTableEntryFlag::Valid)
        } else {
            None
        }
    }

    pub fn write(
        &self,
        addr: u64,
        data: &[u8],
        flags: impl Into<FlagSet<PageTableEntryFlag>> + Clone,
    ) {
        // 把数据写到虚拟内存的指定地方
        let mut offset = addr & 0xFFF;
        let mut copied = 0usize;
        let mut page_count = 0usize;
        unsafe {
            while copied < data.len() {
                if let Some(ppn) =
                    self.ensure_created((addr >> 12) + page_count as u64, flags.clone())
                {
                    let start = (ppn << 12) + offset;
                    let end = if (data.len() - copied) > (0x1000 - offset as usize) {
                        (ppn + 1) << 12
                    } else {
                        start + data.len() as u64 - copied as u64
                    };
                    let ptr = start as *mut u8;
                    for i in 0..(end - start) {
                        ptr.add(i as usize).write(data[copied + i as usize]);
                    }
                    copied += (end - start) as usize;
                    page_count += 1;
                } else {
                    panic!("memory out");
                }
            }
        }
    }

    pub fn satp(&self) -> u64 {
        if let Some(table) = &self.root {
            (8 << 60) | table.page_number()
        } else {
            0
        }
    }

    pub fn free(self) {
        if let Some(table) = self.root {
            table.free();
        }
    }

    #[deprecated]
    pub fn print_page_table(&self) {
        println!("VPN => PPN");
        if let Some(table2) = &self.root {
            for vpn2 in 0..512 {
                let pte2 = table2.entry(vpn2);
                if pte2.is_valid() {
                    if pte2.is_leaf() {
                        // G page
                        println!("invalid page table at {:#x}#{}", table2.page_number(), vpn2);
                    } else {
                        let table1 = pte2.as_page_table(1);
                        for vpn1 in 0..512 {
                            let pte1 = table1.entry(vpn1);
                            if pte1.is_valid() {
                                if pte1.is_leaf() {
                                    println!(
                                        "invalid page table at {:#x}#{}",
                                        table2.page_number(),
                                        vpn1
                                    );
                                } else {
                                    let table0 = pte1.as_page_table(0);
                                    for vpn0 in 0..512 {
                                        let pte0 = table0.entry(vpn0);
                                        if pte0.is_valid() && pte0.is_leaf() {
                                            println!(
                                                "{:#x} => {:#x}",
                                                (vpn2 << 18) + (vpn1 << 9) + vpn0,
                                                pte0.physical_page_number()
                                            );
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
}
