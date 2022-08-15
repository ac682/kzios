use alloc::boxed::Box;
use alloc::collections::VecDeque;

use spin::Mutex;

use crate::println;
use crate::process::{Process, ProcessState};
use crate::trap::TrapFrame;

extern "C" {
    fn _switch_to_user(frame_address: usize, pc: usize);
}

static mut PROCESS_LIST: Mutex<Option<VecDeque<Process>>> = Mutex::new(None);

// frame pointer, pc
fn schedule() -> (usize, usize) {
    // take one process
    unsafe {
        if let Some(mut list) = PROCESS_LIST.lock().as_mut() {
            list.rotate_left(1);
            if let Some(process) = list.front_mut() {
                return match process.state {
                    ProcessState::Idle => {
                        process.state = ProcessState::Running;
                        (&process.trap as *const TrapFrame as usize, process.pc)
                    }
                    ProcessState::Dead => todo!(),
                    ProcessState::Sleeping => schedule(),
                    ProcessState::Running => {
                        (&process.trap as *const TrapFrame as usize, process.pc)
                    }
                };
            }
        }
    }
    (0, 0)
}

pub fn schedule_next_process() {
    unsafe {
        match schedule() {
            (0, 0) => todo!("no process can be done"),
            (trap_pointer, pc) => _switch_to_user(trap_pointer, pc)
        }
    }
}

pub fn add_process(process: Process) {
    unsafe {
        let mut mutex = PROCESS_LIST.lock();
        if let Some(mut list) = mutex.as_mut() {
            list.push_back(process);
        } else {
            let mut list: VecDeque<Process> = VecDeque::new();
            list.push_back(process);
            mutex.replace(list);
        }
    }
}
