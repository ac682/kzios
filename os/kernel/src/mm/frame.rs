use buddy_system_allocator::LockedFrameAllocator;
use erhino_shared::mem::PageNumber;
use spin::Once;

use crate::external::{_kernel_end, _memory_end};

use super::page::PAGE_BITS;

static mut FRAME_ALLOCATOR: Once<LockedFrameAllocator> = Once::new();

pub struct FrameTracker {
    number: PageNumber,
    count: usize,
}

impl FrameTracker {
    pub fn new(number: PageNumber, count: usize) -> Self {
        Self {
            number: number,
            count: count,
        }
    }

    pub fn len(&self) -> usize {
        self.count
    }

    pub fn start(&self) -> PageNumber {
        self.number
    }
}

impl Drop for FrameTracker {
    fn drop(&mut self) {
        dealloc(self.number, self.count)
    }
}

pub fn init() {
    let free_start: usize = _kernel_end as usize >> PAGE_BITS;
    let free_end = _memory_end as usize >> PAGE_BITS;
    unsafe {
        let allocator = LockedFrameAllocator::new();
        allocator.lock().add_frame(free_start, free_end);
        FRAME_ALLOCATOR.call_once(|| allocator);
    }
}

pub fn alloc(count: usize) -> Option<PageNumber> {
    unsafe { FRAME_ALLOCATOR.get_mut().unwrap().lock().alloc(count) }
}

pub fn dealloc(frame: PageNumber, count: usize) {
    unsafe {
        FRAME_ALLOCATOR
            .get_mut()
            .unwrap()
            .lock()
            .dealloc((frame) as usize, count);
    }
}

pub fn borrow(count: usize) -> Option<FrameTracker> {
    if let Some(number) = alloc(count) {
        Some(FrameTracker::new(number, count))
    } else {
        None
    }
}
