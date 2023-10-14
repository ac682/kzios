use core::{cell::OnceCell, mem::size_of};

use buddy_system_allocator::LockedFrameAllocator;
use erhino_shared::mem::PageNumber;

use crate::external::{_frame_start, _memory_end};

use super::page::{PAGE_BITS, PAGE_SIZE};

static mut FRAME_ALLOCATOR: OnceCell<LockedFrameAllocator<32>> = OnceCell::new();

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

pub fn init(start_addr: usize) {
    let free_start: usize = start_addr >> PAGE_BITS;
    let free_end = _memory_end as usize >> PAGE_BITS;
    unsafe {
        let allocator = LockedFrameAllocator::new();
        allocator.lock().add_frame(free_start, free_end);
        let _ = FRAME_ALLOCATOR.set(allocator);
    }
}

pub fn add_frame(start: usize, end: usize) {
    unsafe {
        FRAME_ALLOCATOR
            .get_mut()
            .unwrap()
            .lock()
            .add_frame(start, end)
    }
}

pub fn alloc(count: usize) -> Option<PageNumber> {
    unsafe {
        let ret = FRAME_ALLOCATOR.get_mut().unwrap().lock().alloc(count);
        if let Some(result) = ret {
            let size = count * (PAGE_SIZE / size_of::<u64>());
            let ptr = (result << PAGE_BITS) as *mut u64;
            for i in 0..size {
                ptr.add(i).write(0);
            }
        }
        ret
    }
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
