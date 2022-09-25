use core::slice;

use flagset::FlagSet;
use riscv::{asm::sfence_vma_all, register::satp};

use crate::paged::page_table::PageTableEntryFlag;
use crate::{alloc, println};

use super::page_table::{PageTable, PageTableEntry};

pub struct MemoryUnit {
    root: PageTable,
}

impl MemoryUnit {
    pub fn new(root: PageTable) -> Self {
        Self { root }
    }

    pub fn map(&self, ppn: u64, vpn: u64, flags: impl Into<FlagSet<PageTableEntryFlag>>) {
        self.root
            .map(ppn, vpn, flags.into() | PageTableEntryFlag::Valid);
    }

    pub fn unmap(&self, vpn: u64) {
        self.root.unmap(vpn);
    }

    pub fn fill(&self, vpn: u64, count: usize, flags: impl Into<FlagSet<PageTableEntryFlag>>) {
        let f = flags.into();
        let cnt = match count {
            0 => 1,
            _ => count,
        };
        for i in 0..cnt {
            self.root
                .map(alloc().unwrap(), vpn + i as u64, f)
                .expect("PANIC!");
        }
    }

    pub fn ensure_created(
        &self,
        vpn: u64,
        flags: impl Into<FlagSet<PageTableEntryFlag>>,
    ) -> Option<u64> {
        self.root
            .ensure_created(vpn, flags.into() | PageTableEntryFlag::Valid)
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
        (8 << 60) | self.root.page_number()
    }

    pub fn free(self) {
        self.root.free();
    }

    pub fn fork(&self) -> MemoryUnit {
        let unit = Self::new(PageTable::new(2, alloc().unwrap()));
        self.enumerate(|pte, vpn| {
            unit.write(
                vpn << 12,
                unsafe {
                    core::slice::from_raw_parts(
                        (pte.physical_page_number() << 12) as *const u8,
                        4096,
                    )
                },
                pte.flags(),
            );
        });
        unit
    }

    #[doc(hidden)]
    pub fn enumerate(&self, func: impl Fn(&PageTableEntry, u64)) {
        let table2 = &self.root;
        for vpn2 in 0..512 {
            let pte2 = table2.entry(vpn2);
            if pte2.is_valid() {
                if pte2.is_leaf() {
                    // G page
                    todo!()
                } else {
                    let table1 = pte2.as_page_table(1);
                    for vpn1 in 0..512 {
                        let pte1 = table1.entry(vpn1);
                        if pte1.is_valid() {
                            if pte1.is_leaf() {
                                // M page
                                todo!()
                            } else {
                                let table0 = pte1.as_page_table(0);
                                for vpn0 in 0..512 {
                                    let pte0 = table0.entry(vpn0);
                                    if pte0.is_valid() && pte0.is_leaf() {
                                        func(&pte0, ((vpn2 << 18) + (vpn1 << 9) + vpn0) as u64);
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
    }

    #[doc(hidden)]
    pub fn print_page_table(&self) {
        println!("VPN => PPN");
        self.enumerate(|pte, vpn| {
            println!("{:#x} => {:#x}", vpn, pte.physical_page_number());
        });
    }
}
