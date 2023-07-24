use core::{alloc::Layout, panic::PanicInfo};

use buddy_system_allocator::{Heap, LockedHeapWithRescue};
use erhino_shared::proc::Termination;

use crate::dbg;

#[global_allocator]
static mut HEAP_ALLOCATOR: LockedHeapWithRescue<HEAP_ORDER> =
    LockedHeapWithRescue::new(heap_rescue);

const INITIAL_HEAP_SIZE: usize = 1 * 0x1000;
const HEAP_ORDER: usize = 64;

extern "C" {
    fn _segment_break();
}

#[lang = "start"]
fn lang_start<T: Termination + 'static>(main: fn() -> T, _: isize, _: *const *const u8) -> ! {
    let single = 0x1000;
    unsafe {
        //sys_extend(_segment_break as usize, single, 0b011);
        HEAP_ALLOCATOR.lock().init(_segment_break as usize, single);
    }
    let code = main().to_exit_code();
    unsafe {
        loop {
            // sys_exit(code);
        }
    }
}

#[panic_handler]
fn handle_panic(info: &PanicInfo) -> ! {
    dbg!("Process panicking...\n");
    if let Some(location) = info.location() {
        dbg!(
            "file {}, {}: {}\n",
            location.file(),
            location.line(),
            info.message().unwrap()
        );
    } else {
        dbg!("no information available.\n");
    }
    loop {}
}

fn heap_rescue(heap: &mut Heap<HEAP_ORDER>, layout: &Layout) {
    dbg!("rescue: ");
    let single = 0x1000;
    let mut size = single;
    while layout.size() > size {
        size *= 2;
    }
    let last = heap.stats_total_bytes() + _segment_break as usize;
    dbg!("{:#x}..{:#x}", last, last + size);
    unsafe {
        // sys_extend(last, size, 0b011);
        heap.add_to_heap(last, last + size);
    }
}

#[alloc_error_handler]
fn handle_alloc_error(layout: Layout) -> ! {
    panic!("Heap allocation error, layout = {:?}", layout);
}
