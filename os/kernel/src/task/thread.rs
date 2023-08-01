use alloc::string::String;
use erhino_shared::proc::{Tid, ProcessState};

use crate::trap::TrapFrame;

pub struct Thread{
    pub tid: Tid,
    pub name: String,
    pub frame: TrapFrame,
    pub state: ProcessState,
}