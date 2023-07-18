use core::{
    alloc::Layout,
    hint::spin_loop,
    panic::{self, PanicInfo},
};

use buddy_system_allocator::{Heap, LockedHeapWithRescue};
use dtb_parser::{prop::PropertyValue, traits::HasNamedProperty};
use erhino_shared::proc::Termination;

use crate::{
    external::{_heap_start, _park, _stack_start},
    hart, print, println, sbi, mm::frame::frame_alloc,
};

const HEAP_ORDER: usize = 64;

#[global_allocator]
static mut HEAP_ALLOCATOR: LockedHeapWithRescue<HEAP_ORDER> =
    LockedHeapWithRescue::new(heap_rescue);

static mut ENV_INIT: bool = false;

#[lang = "start"]
fn rust_start<T: Termination + 'static>(main: fn() -> T, hartid: usize, dtb_addr: usize) -> isize {
    if hartid == 0 {
        // rust initialization
        unsafe {
            let heap_start = _heap_start as usize;
            let size = _stack_start as usize - heap_start;
            HEAP_ALLOCATOR.lock().init(heap_start, size);
        }
        early_init(dtb_addr);
        unsafe {
            ENV_INIT = true;
        }
        hart::of_hart(hartid).init();
        println!("Hart #{} init completed, go kernel init", hartid);
        main();
        //hart::send_ipi_all();
    } else {
        unsafe {
            while !ENV_INIT {
                spin_loop();
            }
        }
        hart::of_hart(hartid).init();
        println!("Hart #{} init completed, sleeping", hartid);
        unsafe {
            _park();
        }
    }
    unreachable!();
}

fn early_init(dtb_addr: usize) {
    sbi::init();
    let tree = dtb_parser::device_tree::DeviceTree::from_address(dtb_addr).unwrap();
    // println!("{}", tree);
    let mut clint_base: usize;
    let mut timebase_frequency: usize = 0;
    for node in tree.into_iter() {
        if node.name().starts_with("clint") {
            if let Some(prop) = node.find_prop("reg") {
                if let PropertyValue::Address(address, _size) = prop.value() {
                    clint_base = *address as usize;
                }
            }
        } else if node.name() == "cpus" {
            if let Some(prop) = node.find_prop("timebase-frequency") {
                if let PropertyValue::Integer(frequency) = prop.value() {
                    timebase_frequency = *frequency as usize;
                }
            }
        }
    }
    if timebase_frequency == 0 {
        panic!("device tree provides no cpu information");
    }
    hart::init(timebase_frequency);
}

#[panic_handler]
fn handle_panic(info: &PanicInfo) -> ! {
    if let Some(location) = info.location() {
        println!(
            "\x1b[0;31mKernel panicking at #{}: \x1b[0m\nfile {}, {}: {}",
            hart::context().hartid,
            location.file(),
            location.line(),
            info.message().unwrap()
        );
    } else {
        println!(
            "\x1b[0;31mKernel panicking at #{}: no information available.",
            hart::context().hartid
        );
    }
    loop {}
}

#[alloc_error_handler]
pub fn handle_alloc_error(layout: Layout) -> ! {
    panic!("Heap allocation error, layout = {:?}", layout);
}

fn heap_rescue(heap: &mut Heap<HEAP_ORDER>, layout: &Layout) {
    let single = 4096;
    let mut size = 1;
    unsafe {
        while layout.size() > size * single {
            size *= 2;
        }
        if let Some(frame_start) = frame_alloc(size) {
            heap.add_to_heap(frame_start * single, (frame_start + size) * single);
        } else {
            panic!("kernel memory request but ran out of memory");
        }
    }
}
