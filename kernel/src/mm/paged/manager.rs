use riscv::{asm::sfence_vma_all, register::satp};

use crate::println;

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

    pub fn map(&self, ppn: usize, vpn: usize, count: usize, flags: usize) {
        let cnt = match count {
            0 => 1,
            _ => count,
        };
        if let Some(table) = &self.root {
            for i in 0..cnt {
                table.map(ppn + i, vpn + i, flags).expect("PANIC!");
            }
        }
    }

    pub fn activate(&self) {
        unsafe {
            if let Some(table) = &self.root {
                satp::set(satp::Mode::Sv39, 0, table.page_number());
                sfence_vma_all();
            }
        }
    }

    pub fn print_page_table(&self) {
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
