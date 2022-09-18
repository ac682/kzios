use alloc::{sync::Arc, vec::Vec};
use core::usize;

use spin::{self, Mutex};

use crate::{_kernel_end, _memory_end};

type FrameAllocatorImpl = StackFrameAllocator;

lazy_static! {
    pub static ref FRAME_ALLOCATOR: Mutex<FrameAllocatorImpl> =
        Mutex::new(FrameAllocatorImpl::new());
}

pub fn init() {
    let start = _kernel_end as u64;
    let end = _memory_end as u64;
    FRAME_ALLOCATOR
        .lock()
        .init((start - 1 + 4096) / 4096, end / 4096);
}

pub trait FrameAllocator {
    fn new() -> Self;
    fn alloc(&mut self) -> Option<u64>;
    fn free(&mut self, ppn: u64);
}

pub struct StackFrameAllocator {
    page_number_start: u64,
    page_number_pointer: u64,
    page_number_end: u64,
    recycled: Vec<u64>,
}

impl StackFrameAllocator {
    pub fn init(&mut self, ppn_start: u64, ppn_end: u64) {
        self.page_number_start = ppn_start;
        self.page_number_end = ppn_end;
        self.page_number_pointer = ppn_start;
    }
}

impl FrameAllocator for StackFrameAllocator {
    fn new() -> Self {
        StackFrameAllocator {
            page_number_start: 0,
            page_number_pointer: 0,
            page_number_end: 0,
            recycled: Vec::new(),
        }
    }
    fn alloc(&mut self) -> Option<u64> {
        if self.recycled.is_empty() {
            if self.page_number_pointer < self.page_number_end {
                self.page_number_pointer += 1;
                Some(self.page_number_pointer)
            } else {
                None
            }
        } else {
            self.recycled.pop()
        }
    }
    fn free(&mut self, ppn: u64) {
        if ppn > self.page_number_end || ppn < self.page_number_start {
            // panic!();
        }
        if ppn == self.page_number_pointer {
            self.page_number_pointer -= 1;
        } else {
            if self.recycled.contains(&ppn) {
                // panic!();
            } else {
                self.recycled.push(ppn);
            }
        }
    }
}
