use alloc::collections::VecDeque;
use alloc::vec::Vec;

use riscv::register::{mepc, mscratch};

use crate::process::scheduler::ProcessScheduler;
use crate::process::ProcessState;
use crate::timer::{disable_timers, enable_timers};
use crate::trap::TrapFrame;
use crate::{println, set_next_timer, Process};

extern "C" {
    fn _switch_to_user();
}

pub struct FlatScheduler {
    list: Vec<Process>,
    current: usize,
}

impl FlatScheduler {
    pub fn new() -> Self {
        Self {
            list: Vec::new(),
            current: 0,
        }
    }

    fn set_next_timer(&self) {
        set_next_timer(10_000_0);
    }

    fn index(&mut self, pid: usize) -> Option<&mut Process> {
        self.list.get_mut(pid)
    }

    fn move_next(&mut self) {
        self.current = (self.current + 1) % self.list.len();
    }

    fn mark_dead(&mut self, pid: usize, exit_code: u32) {
        if let Some(proc) = self.index(pid) {
            proc.state = ProcessState::Dead;
            proc.exit_code = exit_code;
        }
    }
}

impl ProcessScheduler for FlatScheduler {
    fn add_process(&mut self, proc: Process) {
        //TODO: 检查当前进程列表,找到一个marked dead进程就将其替换,否则插入到末尾
        self.list.push(proc);
    }

    fn exit_process(&mut self, exit_code: u32) {
        // 关闭时钟, 此时一般位于 ecall trap里,全局中断本来就关着吧?
        disable_timers();
        self.mark_dead(self.current, exit_code);
        self.switch_next();
        enable_timers();
    }

    fn switch_next(&mut self) -> usize {
        let mut do_next = true;
        if let Some(current) = self.current() {
            match current.state {
                ProcessState::Running => {
                    current.pc = mepc::read();
                }
                ProcessState::Idle => {
                    do_next = false;
                }
                _ => (),
            }
        }
        if do_next {
            self.move_next();
        }
        let mut next_pid = 0;
        let mut skip = false;
        if let Some(next) = self.current() {
            match next.state {
                ProcessState::Running | ProcessState::Idle => {
                    next_pid = next.pid;
                    mscratch::write(&next.trap as *const TrapFrame as usize);
                    mepc::write(next.pc);
                    next.state = ProcessState::Running;
                }
                ProcessState::Dead => {
                    skip = true;
                }
                _ => (),
            }
        }
        self.set_next_timer();
        if skip {
            self.switch_next()
        } else {
            next_pid
        }
    }

    // 该处只会被调用一次,且用作内核到用户空间的过渡
    fn switch_to_user(&mut self) {
        // 从 0 号进程开始
        if self.list.len() > 0 {
            self.set_next_timer();
        } else {
            panic!("no process to be switched to");
        }
    }

    fn timer_tick(&mut self) {
        self.switch_next();
    }

    fn current(&mut self) -> Option<&mut Process> {
        self.list.get_mut(self.current)
    }

    fn len(&self) -> usize {
        self.list.len()
    }
}
