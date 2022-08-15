#![no_std]
#![no_main]
#![feature(panic_info_message, alloc_error_handler)]
#![feature(pin_macro)]
#![allow(unused)]

extern crate alloc;
#[macro_use]
extern crate lazy_static;

use core::{arch::global_asm, panic};
use core::arch::asm;
use core::borrow::Borrow;

use spin::Mutex;

use mm::{
    heaped,
    paged::{self, alloc, page_table::PageTable},
};
use primitive::qemu;

use crate::external::{_kernel_end, _kernel_start, _memory_end, _memory_start, _stack_end, _stack_start, _trap_stack_end, _trap_stack_start};
use crate::lang_items::print;
use crate::process::manager::add_process;
use crate::process::Process;
use crate::timer::set_next_timer;

mod lang_items;
mod mm;
mod primitive;
mod process;
mod syscall;
mod trap;
mod pmp;
mod external;
mod timer;

global_asm!(include_str!("assembly.asm"));

#[no_mangle]
extern "C" fn main() -> ! {
    // kernel init
    pmp::init();
    mm::init();
    trap::init();
    timer::init();
    // device init
    qemu::init();
    // hello world
    println!("Hello, World!");
    print_sections();
    let process0 = Process::new_fn(init0);
    let process1 = Process::new_fn(init1);
    add_process(process0);
    add_process(process1);
    set_next_timer();
    unsafe {
        loop {
            asm!("wfi")
        }
    }
}

fn print_sections() {
    let memory_start = _memory_start as usize;
    let kernel_start = _kernel_start as usize;
    let stack_start = _stack_start as usize;
    let stack_end = _stack_end as usize;
    let trap_stack_start = _trap_stack_start as usize;
    let trap_stack_end = _trap_stack_end as usize;
    let kernel_end = _kernel_end as usize;
    let memory_end = _memory_end as usize;

    println!("memory@{:#x}:{:#x}={}K {{", memory_start, memory_end, (memory_end - memory_start) / 1024);
    println!("  kernel@{:#x}:{:#x}={}K {{", kernel_start, kernel_end, (kernel_end - kernel_start) / 1024);
    println!("    stack@{:#x}:{:#x}={}K;", stack_start, stack_end, (stack_end - stack_start) / 1024);
    println!("    trap_stack@{:#x}:{:#x}={}K;", trap_stack_start, trap_stack_end, (trap_stack_end - trap_stack_start) / 1024);
    println!("  }}");
    println!("  user@{:#x}:{:#x}={}K;", kernel_end, memory_end, (memory_end - kernel_end) / 1024);
    println!("}}");
}

#[no_mangle]
fn init0() {
    loop {
        syscall(0, '0' as usize, 0, 0, 0);
    }
}

#[no_mangle]
fn init1() {
    loop {
        syscall(0, '1' as usize, 0, 0, 0);
    }
}

fn syscall(id: usize, arg0: usize, arg1: usize, arg2: usize, arg3: usize) {
    let mut ret = 0usize;
    unsafe { asm!("ecall", inlateout("x10") arg0 => ret, in("x11") arg1, in("x12") arg2, in("x13") arg3, in("x17") id) };
}
