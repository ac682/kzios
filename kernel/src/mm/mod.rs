use alloc::vec::Vec;

use crate::config::MEMORY_END;
use crate::mm::paged::address::PhysicalPageNumber;
use crate::mm::paged::frame_allocator::{frame_alloc, frame_dealloc, print_frame_use};

mod heap_allocator;
pub mod paged;

pub fn init() {
    heap_allocator::init_heap();
    paged::init();
}