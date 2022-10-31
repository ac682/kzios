use core::{alloc::Layout, panic::PanicInfo};

use buddy_system_allocator::{Heap, LockedHeapWithRescue};
use erhino_shared::process::Termination;

use crate::call::{sys_exit, sys_extend};

#[allow(unused)]
const INITIAL_HEAP_SIZE: usize = 1 * 0x1000;
const HEAP_ORDER: usize = 64;

extern "C" {
    fn _segment_break();
}

#[global_allocator]
static HEAP_ALLOCATOR: LockedHeapWithRescue<HEAP_ORDER> = LockedHeapWithRescue::new(heap_rescue);

#[lang = "start"]
fn lang_start<T: Termination + 'static>(main: fn() -> T) -> ! {
    unsafe {
        sys_extend(_segment_break as usize, 4096, 0b011).unwrap();
        HEAP_ALLOCATOR.lock().init(_segment_break as usize, 4096);
    }
    let code = main().to_exit_code();
    unsafe {
        loop {
            sys_exit(code);
        }
    }
}

#[panic_handler]
fn handle_panic(_info: &PanicInfo) -> ! {
    todo!();
}

fn heap_rescue(heap: &mut Heap<HEAP_ORDER>, layout: &Layout) {
    let single = 4096;
    let mut size = single;
    unsafe{
        while layout.size() > size{
            size *= 2;
        }
        let last = heap.stats_total_bytes() + _segment_break as usize;
        sys_extend(last, size, 0b011).unwrap();
        heap.add_to_heap(last, last + size);
    }
}

#[alloc_error_handler]
fn handle_alloc_error(layout: Layout) -> ! {
    panic!("Heap allocation error, layout = {:?}", layout);
}
