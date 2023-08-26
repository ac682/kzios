#![no_std]

use core::hint::spin_loop;

use rinlib::{ipc::signal, preclude::*, shared::proc::SystemSignal};

fn main() {
    debug!("Hello, pm!");
    signal::set_handler(SystemSignal::Notify | SystemSignal::Terminate, handler);
    loop {
        spin_loop()
    }
}

fn handler(signal: SystemSignal) {
    debug!("{:?}", signal);
}
