use core::panic;

use alloc::vec::Vec;

use crate::config::{MEMORY_END, PAGE_SIZE_BITS};
use crate::sync::safe_cell::SafeCell;

use super::address::PhysicalPageNumber;

use lazy_static::*;

trait FrameAllocator {
    fn new() -> Self;
    fn alloc(&mut self) -> Option<PhysicalPageNumber>;
    fn dealloc(&mut self, ppn: PhysicalPageNumber);
}

struct StackAllocator {
    current: PhysicalPageNumber,       // 空闲内存起始页号， 闭区间
    end: PhysicalPageNumber,           // 空闲内存的结束页号， 开区间
    recycled: Vec<PhysicalPageNumber>, // 回收了的页号
}

impl FrameAllocator for StackAllocator {
    fn new() -> Self {
        extern "C" {
            fn ekernel();
        }
        Self {
            current: PhysicalPageNumber::from_address(ekernel as usize),
            end: PhysicalPageNumber::from_address(MEMORY_END),
            recycled: Vec::new(),
        }
    }

    fn alloc(&mut self) -> Option<PhysicalPageNumber> {
        if let Some(ppn) = self.recycled.pop() {
            Some(ppn.into()) // 直接从回收空间里取一个
        } else {
            if self.current == self.end {
                None // 用完了
            } else {
                let res = Some(self.current);
                self.current += 1;
                res
            }
        }
    }

    fn dealloc(&mut self, ppn: PhysicalPageNumber) {
        let num: usize = ppn.into();
        if ppn >= self.current || self.recycled.iter().find(|&v| *v == ppn).is_some() {
            panic!("Frame ppn={:#x?} has not been allocated!", num)
        } else if ppn + 1 == self.current {
            // 刚好就是刚分配出去的页
            self.current -= 1
        } else {
            self.recycled.push(ppn)
        }
    }
}

type FrameAllocatorImpl = StackAllocator;

lazy_static! {
    static ref FRAME_ALLOCATOR: SafeCell<FrameAllocatorImpl> =
        unsafe { SafeCell::new(FrameAllocatorImpl::new()) };
}

pub fn frame_alloc() -> Option<FrameTracker> {
    FRAME_ALLOCATOR
        .exclusive_access()
        .alloc()
        .map(|page_number| FrameTracker::new(page_number))
}

pub fn frame_dealloc(ppn: PhysicalPageNumber) {
    FRAME_ALLOCATOR.exclusive_access().dealloc(ppn);
}

pub fn print_frame_use() {
    println!("======= FRAME USED ======");
    extern "C" {
        fn ekernel();
    }

    let allocator = FRAME_ALLOCATOR.exclusive_access();

    let start = (ekernel as usize) >> PAGE_SIZE_BITS;
    let end: usize = allocator.current.into();
    let mut current = start;

    for frame_number in start..end {
        if allocator.recycled.contains(&(frame_number.into())) {
            if current < frame_number {
                println!(
                    "[{:#x}, {:#x})\t{} frame(s)",
                    &current << PAGE_SIZE_BITS,
                    &frame_number << PAGE_SIZE_BITS,
                    frame_number - &current
                );
            }
            current = frame_number + 1;
        }
    }
    if current < end {
        println!(
            "[{:#x}, {:#x})\t{} frame(s)",
            &current << PAGE_SIZE_BITS,
            &end << PAGE_SIZE_BITS,
            end - &current
        );
    }
    println!("========= USE END ========");
}

pub struct FrameTracker {
    pub page_number: PhysicalPageNumber,
}

impl FrameTracker {
    pub fn new(ppn: PhysicalPageNumber) -> Self {
        let v = Self { page_number: ppn };
        // 初始化ppn所在帧字节全部为0
        let bytes = v.page_number.get_frame();
        for byte in bytes {
            *byte = 0;
        }
        v
    }
}

impl Drop for FrameTracker {
    fn drop(&mut self) {
        frame_dealloc(self.page_number);
    }
}
