use core::{alloc::Layout, panic::PanicInfo};

use buddy_system_allocator::{Heap, LockedHeapWithRescue};
use dtb_parser::{
    prop::PropertyValue,
    traits::{FindPropertyValue, HasNamedProperty},
    DeviceTree,
};
use erhino_shared::{mem::Address, proc::Termination};

use crate::{
    external::{_heap_start, _stack_start},
    fs, hart,
    mm::{
        self,
        frame::{self, alloc},
        page::PAGE_SIZE,
    },
    println, sbi,
};

const HEAP_ORDER: usize = 32;

#[global_allocator]
static mut HEAP_ALLOCATOR: LockedHeapWithRescue<HEAP_ORDER> =
    LockedHeapWithRescue::new(heap_rescue);

#[lang = "start"]
fn rust_start<T: Termination + 'static>(
    main: fn() -> T,
    _: isize,
    argv: *const *const u8,
    _sigpipe: u8,
) -> isize {
    let dtb_addr = argv as usize;
    // rust initialization
    unsafe {
        let heap_start = _heap_start as usize;
        let size = _stack_start as usize - heap_start;
        HEAP_ALLOCATOR.lock().init(heap_start, size);
    }
    early_init(dtb_addr);
    kernel_init();
    main();
    hart::start_all();
    hart::enter_user();
}

pub static mut INITFS: (Address, usize) = (0, 0);

fn early_init(dtb_addr: usize) {
    sbi::init();
    let tree = DeviceTree::from_address(dtb_addr).expect("device tree not available");
    if let Some(chosen) = tree.find_node("/chosen/initfs") {
        if let Some(PropertyValue::Address(addr, len)) = chosen.value("reg") {
            unsafe {
                INITFS = (*addr as usize, *len as usize);
            };
        }
    }
    if let (0, 0) = unsafe { INITFS } {
        panic!("no initfs info");
    } else {
        frame::init(unsafe { INITFS.0 })
    }
    let mut timebase_frequency: usize = 0;
    for node in tree.into_iter() {
        if node.name() == "cpus" {
            if let Some(prop) = node.find_prop("timebase-frequency") {
                if let PropertyValue::Integer(frequency) = prop.value() {
                    timebase_frequency = *frequency as usize;
                }
            }
            for cpu in node.nodes() {
                if let Some(device) = cpu.find_prop("device_type") {
                    if let PropertyValue::String(string) = device.value() {
                        if *string == "cpu" {
                            if let Some(cpuid) = cpu.find_prop("reg") {
                                if let Some(isa_prop) = cpu.find_prop("riscv,isa") {
                                    if let PropertyValue::String(isa) = isa_prop.value() {
                                        if !(*isa).contains("imafdc") {
                                            continue;
                                        }
                                    }
                                }
                                if let PropertyValue::Address(id, _) = cpuid.value() {
                                    if let Some(clock) = cpu.find_prop("clock-frequency") {
                                        if let &PropertyValue::Integer(frequency) = clock.value() {
                                            hart::register(*id as usize, frequency as usize);
                                        }
                                    } else {
                                        if timebase_frequency != 0 {
                                            hart::register(
                                                *id as usize,
                                                timebase_frequency as usize,
                                            );
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
            break;
        }
    }
}

fn kernel_init() {
    mm::init();
    fs::init();
}

#[panic_handler]
fn handle_panic(info: &PanicInfo) -> ! {
    if let Some(location) = info.location() {
        println!(
            "\x1b[0;31mKernel panicking #{} \x1b[0m\nin file {} at line {}: {}",
            hart::hartid(),
            location.file(),
            location.line(),
            info.message().unwrap()
        );
    } else {
        println!(
            "\x1b[0;31mKernel panicking #{}: \x1b[0mno information available.",
            hart::hartid()
        );
    }
    loop {}
}

#[alloc_error_handler]
pub fn handle_alloc_error(layout: Layout) -> ! {
    panic!("Heap allocation error, layout = {:?}", layout);
}

fn heap_rescue(heap: &mut Heap<HEAP_ORDER>, layout: &Layout) {
    let single = 1 * PAGE_SIZE;
    let mut size = 1;
    let required = layout.size();
    unsafe {
        while required > size * single {
            size *= 2;
        }
        if let Some(frame_start) = alloc(size) {
            heap.add_to_heap(frame_start * single, (frame_start + size) * single);
        } else {
            panic!("kernel memory request but ran out of memory");
        }
    }
}
