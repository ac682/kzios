#![no_std]

use core::hint::spin_loop;

use rinlib::{ipc::signal, preclude::*, shared::proc::SystemSignal};

fn main() {
    debug!("Hello, fs!");
    for i in 0..5000{
        spin_loop()
    }
    signal::send(3, SystemSignal::Terminate).expect("what?");
}
