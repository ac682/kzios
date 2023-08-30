#![no_std]

use core::hint::spin_loop;

use rinlib::{ipc::signal, preclude::*, shared::proc::SystemSignal};

fn main() {
    debug!("Hello, pm!");

    signal::set_handler(SystemSignal::Terminate | SystemSignal::Notify, handler);

    loop {
        spin_loop()
    }
}

fn handler(signal: SystemSignal) {
    debug!("Signal: {:?}", signal);
}
