use alloc::string::String;
use erhino_shared::proc::{Tid, ProcessState};

use crate::trap::TrapFrame;

pub struct Thread{
    name: String,
    tid: Tid,
    frame: TrapFrame,
    state: ProcessState,
}