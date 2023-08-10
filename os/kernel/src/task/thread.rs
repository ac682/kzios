use alloc::{string::String, borrow::ToOwned};
use erhino_shared::proc::{Tid, ExecutionState};

use crate::trap::TrapFrame;

pub struct Thread{
    pub name: String,
    pub state: ExecutionState,
}

impl Thread{
    pub fn new(name: &str) -> Self{
        Self{
            name: name.to_owned(),
            state: ExecutionState::Ready
        }
    }
}