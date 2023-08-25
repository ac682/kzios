use core::{alloc::Layout, arch::asm, panic::PanicInfo};

use buddy_system_allocator::{Heap, LockedHeapWithRescue};
use erhino_shared::proc::Termination;

use crate::{
    call::{sys_exit, sys_extend},
    debug,
};

const INITIAL_HEAP_SIZE: usize = 1 * 0x1000;
const HEAP_ORDER: usize = 64;

#[global_allocator]
static mut HEAP_ALLOCATOR: LockedHeapWithRescue<HEAP_ORDER> =
    LockedHeapWithRescue::new(heap_rescue);

#[lang = "start"]
fn lang_start<T: Termination + 'static>(
    main: fn() -> T,
    _: isize,
    _: *const *const u8,
    _: u8,
) -> isize {
    unsafe {
        let offset = sys_extend(INITIAL_HEAP_SIZE).expect("the first extend call failed");
        HEAP_ALLOCATOR
            .lock()
            .init(offset - INITIAL_HEAP_SIZE, INITIAL_HEAP_SIZE);
    }
    let code = main().to_exit_code();
    unsafe {
        loop {
            sys_exit(code).expect("this can't be wrong");
        }
    }
}

#[panic_handler]
fn handle_panic(info: &PanicInfo) -> ! {
    if let Some(location) = info.location() {
        debug!(
            "Panicking: file {}, {}: {}",
            location.file(),
            location.line(),
            info.message().unwrap()
        );
    } else {
        debug!("Panicking: no information available.");
    }
    loop {}
}

fn heap_rescue(heap: &mut Heap<HEAP_ORDER>, layout: &Layout) {
    debug!("rescue: ");
    let owned = heap.stats_total_bytes();
    let mut size = owned;
    while layout.size() > size {
        size *= 2;
    }
    unsafe {
        let call = sys_extend(size);
        match call {
            Ok(position) => heap.add_to_heap(position - size, position),
            Err(err) => panic!(
                "cannot request more memory region by extend sys call{:?}",
                err
            ),
        }
    }
}

#[alloc_error_handler]
fn handle_alloc_error(layout: Layout) -> ! {
    panic!("Heap allocation error, layout = {:?}", layout);
}
