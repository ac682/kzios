use erhino_shared::proc::{Pid, Signal, SystemSignal};

use crate::{
    call::{sys_signal_send, sys_signal_set},
    rt::{signal_handler, SIGNAL_HANDLER},
};

pub fn set_handler(mask: Signal, handler: fn(Signal)) {
    unsafe {
        SIGNAL_HANDLER = Some(handler);
        sys_signal_set(signal_handler as usize, mask);
    }
}

pub fn send(pid: Pid, signal: Signal) {
    unsafe { sys_signal_send(pid, signal) };
}
