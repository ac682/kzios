use alloc::{format, string::String};
use core::{arch::asm, panic::PanicInfo};
use erhino_shared::process::Termination;
use riscv::register::misa;

use crate::{
    external::{
        _bss_end, _bss_start, _hart_num, _kernel_end, _memory_end, _memory_start, _stack_start,
    },
    mm, peripheral, pmp, print, println,
    proc::pm,
};

const LOGO: &str = include_str!("../logo.txt");

// only #0 goes here, others only called in trap context
#[lang = "start"]
fn rust_start<T: Termination + 'static>(main: fn() -> T, hartid: usize) -> isize {
    // boot stage #0: enter rust environment
    // boot stage #1: hart(core & trap context) initialization
    // 👆 both done in _start@assembly.asm
    // boot stage #2: board(memory & peripheral) initialization
    unsafe {
        board_init();
    }
    pmp::init();
    mm::init();
    pm::init();
    println!("{}\nis still booting", LOGO);
    print_isa();
    print_segments();
    main();
    panic!("unreachable here");
    // do board clean
}

fn print_isa() {
    let isa = misa::read().unwrap();
    let xlen = isa.mxl();
    let mut isa_str = String::new();
    isa_str.push_str(&format!(
        "RV{}",
        match xlen {
            misa::MXL::XLEN32 => "32",
            misa::MXL::XLEN64 => "64",
            misa::MXL::XLEN128 => "128",
        }
    ));
    let bits = isa.bits() & 0x3FF_FFFF;
    for i in 0..26 {
        if (bits >> i) & 1 == 1 {
            isa_str.push((b'A' + i) as char);
        }
    }
    println!("ISA: {}", isa_str);
}

fn print_segments() {
    let stack_per_hart = (_kernel_end as usize - _stack_start as usize) / _hart_num as usize;
    println!(
        "memory@{:#x}..{:#x} {{",
        _memory_start as usize, _memory_end as usize
    );
    println!(
        "\tkernel@{:#x}..{:#x} {{",
        _memory_start as usize, _kernel_end as usize
    );
    println!(
        "\t\tbss@{:#x}..{:#x};",
        _bss_start as usize, _bss_end as usize
    );
    println!("\t\tstack@{:#x}..{:#x} {{", _stack_start as usize, _kernel_end as usize);
    for i in 0..(_hart_num as usize) {
        println!(
            "\t\t\thart{}@{:#x}..{:#x};",
            i, _kernel_end as usize - stack_per_hart * (i + 1), _kernel_end as usize - stack_per_hart * i
        );
    }
    println!("\t\t}}");
    println!("\t}}");
    println!("}}");
}

#[panic_handler]
fn handle_panic(info: &PanicInfo) -> ! {
    print!("Kernel panicking: ");
    if let Some(location) = info.location() {
        println!(
            "file {}, {}: {}",
            location.file(),
            location.line(),
            info.message().unwrap()
        );
    } else {
        println!("no information available.");
    }
    park();
}

pub fn park() -> ! {
    unsafe {
        loop {
            asm!("wfi");
        }
    }
}

extern "Rust" {
    #[linkage = "extern_weak"]
    pub fn board_init();
}
