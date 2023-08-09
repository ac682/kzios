use alloc::string::String;
use erhino_shared::proc::{Tid, ExecutionState};

use crate::trap::TrapFrame;

pub struct Thread{
    pub tid: Tid,
    pub name: String,
    pub state: ExecutionState,
}