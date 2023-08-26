use erhino_shared::{
    call::SystemCallError,
    proc::{Pid, SystemSignal},
};
use flagset::FlagSet;

use crate::call::{sys_signal_send, sys_signal_set};

#[derive(Debug)]
pub enum SignalError {
    InternalError,
    ProcessNotFound,
}

pub fn set_handler<S: Into<FlagSet<SystemSignal>>>(mask: S, handler: fn(SystemSignal)) {
    unsafe {
        let flags: FlagSet<SystemSignal> = mask.into();
        sys_signal_set(flags.bits(), crate::rt::set_signal_handler(handler)).expect("wont failed");
    }
}

pub fn send(pid: Pid, signal: SystemSignal) -> Result<bool, SignalError> {
    unsafe {
        let flag: FlagSet<SystemSignal> = signal.into();
        sys_signal_send(pid, flag.bits()).map_err(|e| match e {
            SystemCallError::ObjectNotFound => SignalError::ProcessNotFound,
            _ => SignalError::InternalError,
        })
    }
}
