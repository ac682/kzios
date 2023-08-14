use erhino_shared::{proc::ExecutionState, mem::Address};

pub struct Thread {
    pub entry_point: Address,
    pub state: ExecutionState,
}

impl Thread {
    pub fn new(entry: Address) -> Self {
        Self {
            entry_point: entry,
            state: ExecutionState::Ready,
        }
    }
}
