use alloc::boxed::Box;
use alloc::collections::VecDeque;

use riscv::register::mscratch;
use spin::Mutex;

use crate::println;
use crate::process::{Process, ProcessState};
use crate::trap::TrapFrame;

static mut PROCESS_LIST: Mutex<Option<VecDeque<Process>>> = Mutex::new(None);

pub fn schedule_next_process(current_pc: usize) -> usize {
    let mut res = 0usize;
    if let Some(mut list) = unsafe { PROCESS_LIST.lock().as_mut() } {
        if let Some(current) = list.front_mut() {
            match current.state {
                ProcessState::Running => {
                    current.pc = current_pc;
                }
                _ => {}
            }
        }
        list.rotate_left(1);
        let mut removable = false;
        if let Some(next) = list.front_mut() {
            match next.state {
                ProcessState::Idle => {
                    next.state = ProcessState::Running;
                    mscratch::write(&next.trap as *const TrapFrame as usize);
                    res = next.pc;
                }
                ProcessState::Dead => {
                    removable = true; // 用最后一个元素替换掉当前这个, 而最后一个元素恰巧就是正在运行的进程
                    res = current_pc;
                }
                ProcessState::Sleeping => todo!(),
                ProcessState::Running => {
                    mscratch::write(&next.trap as *const TrapFrame as usize);
                    res = next.pc;
                }
            }
        }
        if list.len() == 1{
            // this wont happen for init0 is never ended
            res = 0;
        }else {
            list.swap_remove_back(0);
        }
    }
    res
}

pub fn mark_dead(pid: usize) {
    unsafe {
        let mut mutex = PROCESS_LIST.lock();
        if let Some(list) = mutex.as_mut() {
            if let Some(current) = list.front_mut() {
                current.state = ProcessState::Dead;
            }
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
