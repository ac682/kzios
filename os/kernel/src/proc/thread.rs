use alloc::string::String;
use erhino_shared::proc::Tid;

use crate::trap::TrapFrame;

pub struct Thread {
    name: String,
    tid: Tid,
    frame: TrapFrame,
}

impl Thread {
    // tid is assigned when attached to a process
    pub fn new(name: String) -> Self {
        Self {
            name,
            tid: 0,
            frame: TrapFrame::new(),
        }
    }
}
