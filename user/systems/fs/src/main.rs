#![no_std]

use core::arch::asm;

use rinlib::{
    proc::{fork, exit},
    shared::proc::{ProcessPermission, SystemSignal, Signal},
    signal,
};

mod fs;
mod impls;

extern crate rinlib;

fn main() {
    signal::set_handler(SystemSignal::Interrupt as Signal, handle_signal);
    let pid = fork(ProcessPermission::Invalid).unwrap();
    if pid != 0 {
        debug(pid as usize);
        signal::send(pid, SystemSignal::Interrupt as Signal);
    } else {
        loop {}
    }
}

fn handle_signal(signal: Signal) {
    debug(signal as usize);
    exit(0);
}

fn debug(sym: usize) {
    unsafe {
        asm!("ecall", in("x10") sym, in("x17") 0);
    }
}
