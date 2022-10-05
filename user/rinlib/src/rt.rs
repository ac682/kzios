use core::{alloc::Layout, panic::PanicInfo};

use buddy_system_allocator::{LockedHeapWithRescue, Heap};

const INITIAL_HEAP_SIZE: usize = 1 * 0x1000;
const HEAP_ORDER: usize = 64;

#[global_allocator]
static HEAP_ALLOCATOR: LockedHeapWithRescue<HEAP_ORDER> = LockedHeapWithRescue::new(heap_rescue);


#[lang = "start"]
fn lang_start<T: 'static>(
    main: fn() -> T,
    _argc: isize,
    _argv: *const *const u8,
) -> isize {
    // init heap
    main();
    0
}

#[panic_handler]
fn handle_panic(_info: &PanicInfo) -> ! {
    todo!();
}

fn heap_rescue(_heap: &mut Heap<HEAP_ORDER>, _layout: &Layout){
    todo!();
}

#[alloc_error_handler]
fn handle_alloc_error(layout: Layout) -> ! {
    panic!("Heap allocation error, layout = {:?}", layout);
}

