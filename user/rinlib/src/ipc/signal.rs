use erhino_shared::{
    call::SystemCallError,
    proc::{Pid, SignalMap, SystemSignal},
};
use flagset::FlagSet;
use num_traits::{FromPrimitive, ToPrimitive};

use crate::call::{sys_signal_send, sys_signal_set, sys_signal_return};

static mut SIGNAL_HANDLER: Option<fn(SystemSignal)> = None;

#[derive(Debug)]
pub enum SignalError {
    InternalError,
    ProcessNotFound,
}

pub fn set_handler<S: Into<FlagSet<SystemSignal>>>(mask: S, handler: fn(SystemSignal)) {
    unsafe {
        let flags: FlagSet<SystemSignal> = mask.into();
        SIGNAL_HANDLER = Some(handler);
        sys_signal_set(flags.bits(), signal_handler_wrapper as usize).expect("this wont failed");
    }
}

pub fn send(pid: Pid, signal: SystemSignal) -> Result<bool, SignalError> {
    unsafe {
        sys_signal_send(
            pid,
            signal
                .to_u64()
                .expect("cast system signal to signal map wont failed"),
        )
        .map_err(|e| match e {
            SystemCallError::ObjectNotFound => SignalError::ProcessNotFound,
            _ => SignalError::InternalError,
        })
    }
}

fn signal_handler_wrapper(map: SignalMap) {
    if let Some(handler) = unsafe { SIGNAL_HANDLER } {
        if let Some(signal) = SystemSignal::from_u64(map) {
            handler(signal)
        }
    }
    unsafe {
        sys_signal_return().expect("wont failed if signal_handler called only by kernel");
    }
}
