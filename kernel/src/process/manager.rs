use crate::process::{Process, ProcessState};
use alloc::boxed::Box;
use alloc::collections::VecDeque;
use spin::Mutex;

static mut PROCESS_LIST: Mutex<Option<VecDeque<Process>>> = Mutex::new(None);

pub fn schedule() {
    // take one process
    unsafe {
        if let Some(mut list) = PROCESS_LIST.lock().take() {
            list.rotate_left(1);
            if let Some(mut process) = list.front() {
                match process.state {
                    ProcessState::Idle => todo!(),
                    ProcessState::Dead => todo!(),
                    ProcessState::Sleeping => {
                        todo!()
                        // tail-recurse schedule the next right now
                    }
                    ProcessState::Running => {
                        todo!();
                        // do process and set next timer
                    }
                };
            }
        }
    }
}
