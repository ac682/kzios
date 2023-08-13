use alloc::{borrow::ToOwned, string::String};
use erhino_shared::proc::ExecutionState;

pub struct Thread {
    pub name: String,
    pub state: ExecutionState,
}

impl Thread {
    pub fn new(name: &str) -> Self {
        Self {
            name: name.to_owned(),
            state: ExecutionState::Ready,
        }
    }
}
