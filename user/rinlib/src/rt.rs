use core::{alloc::Layout, panic::PanicInfo};
use erhino_shared::proc::{Pid, SystemSignal, Termination};
use erhino_shared::sync::spin::SimpleLock;
use talc::{OomHandler, Span, Talc, Talck};

use crate::call::sys_extend;
use crate::env;
use crate::{call::sys_exit, debug, ipc::signal};

const INITIAL_HEAP_SIZE: usize = 1 * 0x1000;

struct HeapRecuse {
    heap: Span,
}

impl HeapRecuse {
    const fn new() -> Self {
        Self {
            heap: Span::empty(),
        }
    }
}

impl OomHandler for HeapRecuse {
    fn handle_oom(talc: &mut Talc<Self>, layout: Layout) -> Result<(), ()> {
        let mut count = 1;
        let single = 4096;
        while count * single < layout.size() {
            count *= 2;
        }
        let old = talc.oom_handler.heap;
        if let Ok(offset) = unsafe { sys_extend(count) } {
            let size = count * single;
            let new = if old.is_empty() {
                Span::new((offset - size) as *mut u8, offset as *mut u8)
            } else {
                old.extend(0, size)
            };
            unsafe {
                talc.oom_handler.heap = talc.extend(old, new);
            }
            Ok(())
        } else {
            Err(())
        }
    }
}

#[global_allocator]
static mut HEAP_ALLOCATOR: Talck<SimpleLock, HeapRecuse> = Talc::new(HeapRecuse::new()).lock();

#[lang = "start"]
fn lang_start<T: Termination + 'static>(
    main: fn() -> T,
    argc: isize,
    argv: *const *const u8,
    _sigpipe: u8,
) -> isize {
    let pid = argc as usize as Pid;
    let parent = argv as usize as Pid;
    unsafe {
        env::PID.set(pid).unwrap();
        env::PARENT_PID.set(parent).unwrap();
        let mut talc = HEAP_ALLOCATOR.lock();
        if let Ok(offset) = sys_extend(INITIAL_HEAP_SIZE) {
            let start = offset - INITIAL_HEAP_SIZE;
            if let Ok(heap) = talc.claim(Span::from_base_size(start as *mut u8, INITIAL_HEAP_SIZE))
            {
                talc.oom_handler.heap = heap;
            } else {
                panic!();
            }
        } else {
            panic!();
        }
    }
    signal::set_handler(SystemSignal::Terminate, default_signal_handler);
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
            "Panicking in {} at line {}: {}",
            location.file(),
            location.line(),
            info.message().unwrap()
        );
    } else {
        debug!("Panicking: no information available.");
    }
    unsafe {
        loop {
            sys_exit(-1).expect("this can't be wrong");
        }
    }
}

#[alloc_error_handler]
fn handle_alloc_error(layout: Layout) -> ! {
    panic!("Heap allocation error, layout = {:?}", layout);
}

fn default_signal_handler(signal: SystemSignal) {
    match signal {
        SystemSignal::Terminate => unsafe {
            sys_exit(1).expect("no wish to die");
        },
        _ => {}
    };
}
