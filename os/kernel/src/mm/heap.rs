use buddy_system_allocator::LockedHeap;

use crate::external::{_heap_start, _separator};

const HEAP_ORDER: usize = 64;

#[global_allocator]
static mut HEAP_ALLOCATOR: LockedHeap<HEAP_ORDER> = LockedHeap::empty();

#[alloc_error_handler]
pub fn handle_alloc_error(layout: core::alloc::Layout) -> ! {
    panic!("Heap allocation error, layout = {:?}", layout);
}

pub fn init() {
    unsafe {
        let heap_start = _heap_start as usize;
        let size = _separator as usize - heap_start;
        HEAP_ALLOCATOR.lock().init(heap_start, size);
    }
}
