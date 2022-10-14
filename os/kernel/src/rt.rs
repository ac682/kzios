use alloc::{format, string::String};
use core::{arch::asm, panic::PanicInfo};
use erhino_shared::process::Termination;
use riscv::register::misa;

use crate::{mm, peripheral, pmp, print, println, proc::pm};

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
