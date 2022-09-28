use buddy_system_allocator::LockedHeap;

use crate::process::Termination;
use core::{alloc::Layout, arch::global_asm};

global_asm!(include_str!("rt.asm"));

const INITIAL_HEAP_SIZE: usize = 8 * 0x1000;
const HEAP_ORDER: usize = 64;

#[global_allocator]
static HEAP_ALLOCATOR: LockedHeap<HEAP_ORDER> = LockedHeap::empty();

extern "C" {
    fn _segment_break();
}

#[alloc_error_handler]
pub fn handle_alloc_error(layout: Layout) -> ! {
    // TODO: mmap more
    panic!("Heap allocation error, layout = {:?}", layout);
}

#[export_name = "lang_start"]
#[lang = "start"]
fn lang_start<T: Termination + 'static>(
    main: fn() -> T,
    _argc: isize,
    _argv: *const *const u8,
) -> isize {
    // init heap
    unsafe {
        HEAP_ALLOCATOR
            .lock()
            .init(_segment_break as usize, INITIAL_HEAP_SIZE);
    }
    main().to_exit_code()
}
