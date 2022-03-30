use core::mem::size_of;

use alloc::vec;
use alloc::vec::Vec;

use crate::config::{PAGE_SIZE, PAGE_TABLE_SIZE};

use super::{
    address::{PhysicalAddress, PhysicalPageNumber, VirtualAddress},
    frame_allocator::{frame_alloc, frame_dealloc},
};

/// 64 bits(u64)
/// 64-54           53-28       27-19   18-10   9-8 7 6 5 4 3 2 1 0
/// Reserved(10)    PPN2(26)    PPN1(9) PPN0(9) RSW D A G U X W R V
/// PPN(44): 指示下一级页表所在物理页号，PPN * PAGE_SIZE + VPN(页表级数) * PTE_SIZE 就可以定位到下一级的 PTE
/// RSW: Reserved for Supervisor Software
/// D: Dirty 处理器记录自从页表项上的这一位被清零之后，页表项的对应虚拟页面是否被修改过；
/// A: Accessed 处理器记录自从页表项上的这一位被清零之后，页表项的对应虚拟页面是否被访问过；
/// G: Global 不知道干嘛的；
/// U: User 控制索引到这个页表项的对应虚拟页面是否在 CPU 处于 U 特权级的情况下是否被允许访问；
/// XWR, X(Executable), W(Writeable), R(Readable): 分别控制索引到这个页表项的对应虚拟页面是否允许读/写/执行；
/// Valid: 仅当位 V 为 1 时，页表项才是合法的；
#[repr(C)]
struct PageTableEntry(u64);

impl PageTableEntry {
    pub fn is_valid(&self) -> bool {
        self.0 & 0b1 != 0
    }

    pub fn is_leaf(&self) -> bool {
        self.0 & 0b1110 != 0
    }

    pub fn get_page_number(&self) -> PhysicalPageNumber {
        PhysicalPageNumber::from(self.0 >> 10 & 0xFFF_FFFF_FFFF)
    }

    pub fn get_flags(&self) -> u16 {
        (self.0 & 0b11_1111_1111) as u16
    }

    pub fn set(&mut self, v: u64) {
        self.0 = v;
    }

    pub fn set_as_page_table(&mut self, table_ppn: u64, level: u8) -> PageTable {
        self.set((table_ppn << 10) | 0b0001);
        self.as_page_table(level)
    }

    pub fn link(&mut self, ppn: PhysicalPageNumber, flags: u16) {
        self.0 = (u64::from(ppn) << 10) | (flags & 0b11_1111_1111) as u64;
    }

    pub fn unlink(&mut self) {
        self.0 = 0;
    }

    pub fn as_page_table(&self, level: u8) -> PageTable {
        PageTable::new(level, self.get_page_number())
    }
}

pub struct PageTable {
    //entries: [PageTableEntry; PAGE_SIZE  as usize / size_of::<PageTableEntry>()], // 一个页表占用 4kb， 用来存放512个页表项（由于4k对齐，其本身可以被放进一个页内
    level: u8, // 当前表的级数
    location: PhysicalPageNumber,
}

impl PageTable {
    pub fn new(level: u8, location: PhysicalPageNumber) -> Self {
        Self {
            level: level,
            location: location, // 自己会占用一个帧
        }
    }

    pub fn map(&self, va: VirtualAddress, pa: PhysicalAddress, flags: u16) {
        let pte = self.lookup(va);
        if !pte.is_valid() {
            if self.level != 0 {
                // 没到叶表, 创建枝表
                let frame = frame_alloc().unwrap();
                let table = pte.set_as_page_table(u64::from(frame), self.level - 1);
                table.map(va, pa, flags);
            } else {
                pte.link(PhysicalPageNumber::from(pa), flags | 0b1);
            }
        } else {
            if pte.is_leaf() {
                // leaf
                //TODO: 如果是重复映射不管他，如果是覆盖映射则报错
                panic!(
                    "map a mapped entry va={:#x} pa={:#x} with pte={:#x}",
                    u64::from(va),
                    u64::from(pa),
                    pte.0
                );
            } else {
                // branch
                //TODO: 自己就是0级，那报错
                let branch = pte.as_page_table(self.level - 1);
                branch.map(va, pa, flags);
            }
        }
    }

    pub fn unmap(&self, va: VirtualAddress) {
        let pte = self.lookup(va);
        if pte.is_leaf() {
            //TODO: pte 本身就是非法的那也应该报错
            pte.unlink();
        } else {
            let branch = pte.as_page_table(self.level - 1);
            branch.unmap(va);
        }
    }

    /// 查找对应位置的 PageTableEntry
    /// 如果无效就是未创建
    fn lookup(&self, va: VirtualAddress) -> &mut PageTableEntry {
        let vpn = va.get_page_number();
        let index = match self.level {
            2 => vpn.2,
            1 => vpn.1,
            _ => vpn.0,
        };
        PhysicalAddress::from(self.location)
            .get_mut_offset::<PageTableEntry>((index as usize * size_of::<PageTableEntry>()) as u64)
    }

    fn as_pte_array(&self) -> &[PageTableEntry; PAGE_TABLE_SIZE] {
        self.location.get_mut::<[PageTableEntry; PAGE_TABLE_SIZE]>()
    }

    pub fn free(&self) {
        // 递归 frame dealloc
        let entries = self.as_pte_array();
        for entry in entries {
            if entry.is_valid() {
                if entry.is_leaf() {
                    frame_dealloc(entry.get_page_number());
                } else {
                    entry.as_page_table(self.level - 1).free();
                }
            }
        }
    }
}

/// 只有根页表可以使用的映射，包括等值映射, 效率比依次 root_table.map 高
pub fn map(
    root: &PageTable,
    va_start: VirtualAddress,
    pa_start: PhysicalAddress,
    num_frame: usize,
    flags: u16,
) {
    const MEGA_SIZE: usize = PAGE_TABLE_SIZE;
    const GIGA_SIZE: usize = PAGE_TABLE_SIZE * MEGA_SIZE;

    let (vpn2, vpn1, vpn0) = va_start.get_page_number();
    let ppn = u64::from(PhysicalPageNumber::from(pa_start));

    if false && (num_frame > GIGA_SIZE) {
        // 吉页, 直接在根表上操作
        let entries = root.as_pte_array();
        let pte = &entries[vpn2 as usize];
        // 对其到大叶， 因为 没有 giga_frame allocator 所以没法实现
    } else if false && (num_frame > MEGA_SIZE) {
        // 巨页， 同理实现不了
    } else {
        // 普通页
        //TODO: 我是笨比不会写，用垃圾方法凑合，打个 todo 以后搞手动分配，让其效率高起来
        for cnt in 0..num_frame {
            root.map(
                VirtualAddress::from(u64::from(va_start) + (cnt * PAGE_SIZE) as u64),
                PhysicalAddress::from(u64::from(pa_start) + (cnt * PAGE_SIZE) as u64),
                flags,
            );
        }
    }
}
