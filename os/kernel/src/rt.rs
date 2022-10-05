use alloc::{format, string::String};
use core::{arch::asm, panic::PanicInfo};
use erhino_shared::process::Termination;
use riscv::register::misa;

use crate::{mm, pmp, print, println, trap};

#[lang = "start"]
fn rust_start<T: Termination + 'static>(
    main: fn() -> T
) -> isize {
    unsafe {
        board_init();
    }
    pmp::init();
    mm::init();
    trap::init();
    println!("boot stage #2: board initialization");
    print_isa();
    main();
    println!("unreachable here");
    unsafe {
        loop {
            asm!("wfi");
        }
    }
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
            isa_str.push(('A' as u8 + i) as char);
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