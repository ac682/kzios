#![no_std]

use core::arch::asm;

use rinlib::{
    call::{sys_fork, sys_yield, sys_signal_send},
    process::{Signal, SignalMap},
    signal,
};

mod fs;
mod impls;

extern crate rinlib;

fn main() {
    signal::set_handler(Signal::Interrupt as SignalMap, signal_handler);
    unsafe {
        let pid = sys_fork(0).unwrap();
        if pid != 0 {
            sys_signal_send(pid, Signal::Interrupt);
        }
    }
}

fn signal_handler(signal: Signal) {
    unsafe { asm!("ebreak", in("x10") signal as usize) };
}
