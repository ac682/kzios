use core::{alloc::Layout, panic::PanicInfo};

use buddy_system_allocator::{LockedHeapWithRescue, Heap};
use erhino_shared::proc::Termination;

const HEAP_ORDER: usize = 64;

#[global_allocator]
static mut HEAP_ALLOCATOR: LockedHeapWithRescue<HEAP_ORDER> =
    LockedHeapWithRescue::new(heap_rescue);

const LOGO: &str = include_str!("../logo.txt");

#[lang = "start"]
fn rust_start<T: Termination + 'static>(main: fn() -> T, _hartid: usize) -> isize {
    panic!()
}

#[panic_handler]
fn handle_panic(info: &PanicInfo) -> ! {
    // print!(
    //     "\x1b[0;31mKernel panicking at #{}: \x1b[0m",
    //     mhartid::read()
    // );
    // if let Some(location) = info.location() {
    //     println!(
    //         "file {}, {}: {}",
    //         location.file(),
    //         location.line(),
    //         info.message().unwrap()
    //     );
    // } else {
    //     println!("no information available.");
    // }
    panic!()
}

#[alloc_error_handler]
pub fn handle_alloc_error(layout: Layout) -> ! {
    panic!("Heap allocation error, layout = {:?}", layout);
}

fn heap_rescue(heap: &mut Heap<HEAP_ORDER>, layout: &Layout) {
    // let single = 4096;
    // let mut size = 1;
    // unsafe {
    //     while layout.size() > size * single {
    //         size *= 2;
    //     }
    //     if let Some(frame_start) = frame_alloc(size) {
    //         heap.add_to_heap(frame_start * single, (frame_start + size) * single);
    //     } else {
    //         panic!("kernel memory request but ran out of memory");
    //     }
    // }
}