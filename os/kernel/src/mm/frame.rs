use buddy_system_allocator::FrameAllocator;
use erhino_shared::mem::PageNumber;
use hashbrown::HashMap;
use spin::Once;

use crate::{
    external::{_kernel_end, _memory_end},
    sync::{
        hart::{HartLock, HartReadWriteLock},
        InteriorLock, InteriorReadWriteLock,
    },
};

static mut FRAME_ALLOCATOR: Once<FrameAllocator> = Once::new();
static mut FRAME_LOCK: HartLock = HartLock::new();

static mut TRACKED_PAGES: Once<HashMap<PageNumber, usize>> = Once::new();
static mut TRACKED_LOCK: HartReadWriteLock = HartReadWriteLock::new();

pub fn init() {
    let free_start = _kernel_end as usize >> 12;
    let free_end = _memory_end as usize >> 12;
    unsafe {
        FRAME_ALLOCATOR.call_once(|| FrameAllocator::new());
        FRAME_ALLOCATOR
            .get_mut()
            .unwrap()
            .add_frame(free_start, free_end);
    };
    unsafe {
        TRACKED_PAGES.call_once(|| HashMap::new());
    }
}

pub fn cow_track(ppn: PageNumber, initial: usize) {
    unsafe { TRACKED_LOCK.lock_mut() };
    let tracked = unsafe { TRACKED_PAGES.get_mut().unwrap() };
    tracked
        .entry(ppn)
        .and_modify(|e| *e += 1)
        .or_insert(initial);
    unsafe { TRACKED_LOCK.unlock() };
}

pub fn cow_free(ppn: PageNumber) {
    unsafe { TRACKED_LOCK.lock_mut() };
    let tracked = unsafe { TRACKED_PAGES.get_mut().unwrap() };
    let count = tracked.get(&ppn).unwrap();
    if count == &1 {
        tracked.remove_entry(&ppn);
        frame_dealloc(ppn, 1);
    } else {
        tracked.entry(ppn).and_modify(|e| *e -= 1);
    }
    unsafe { TRACKED_LOCK.unlock() };
}

pub fn cow_usage(ppn: PageNumber) -> usize {
    unsafe { TRACKED_LOCK.lock() };
    let tracked = unsafe { TRACKED_PAGES.get_unchecked() };
    let count = tracked.get(&ppn).unwrap();
    unsafe { TRACKED_LOCK.unlock() };
    *count
}

pub fn frame_alloc(count: usize) -> Option<PageNumber> {
    unsafe {
        FRAME_LOCK.lock();
        let allocator = FRAME_ALLOCATOR.get_mut().unwrap();
        let frame = allocator.alloc(count);
        FRAME_LOCK.unlock();
        frame
    }
}

pub fn frame_dealloc(frame: PageNumber, count: usize) {
    unsafe {
        FRAME_LOCK.lock();
        let allocator = FRAME_ALLOCATOR.get_mut().unwrap();
        allocator.dealloc((frame) as usize, count);
        FRAME_LOCK.unlock();
    }
}
