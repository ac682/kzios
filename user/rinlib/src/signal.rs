use erhino_shared::process::{SignalMap, Signal};

use crate::{call::sys_signal_set, rt::{signal_handler, SIGNAL_HANDLER}};

pub fn set_handler(mask: SignalMap, handler: fn(Signal)){
    unsafe{
        SIGNAL_HANDLER = Some(handler);
        sys_signal_set(signal_handler as usize, mask);
    }
}

