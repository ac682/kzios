use alloc::collections::VecDeque;
use alloc::vec::Vec;

use riscv::register::{mepc, mscratch};

use crate::process::scheduler::ProcessScheduler;
use crate::process::{Address, ExitCode, Pid, ProcessState};
use crate::timer::{disable_timers, enable_timers};
use crate::trap::TrapFrame;
use crate::{println, set_next_timer, timer, Process};

pub struct FlatScheduler {
    list: Vec<Process>,
    current: Pid,
}

impl FlatScheduler {
    pub fn new() -> Self {
        Self {
            list: Vec::new(),
            current: 0,
        }
    }

    fn set_next_timer(&self) {
        enable_timers();
        set_next_timer(10_000_0);
    }

    fn index(&mut self, pid: Pid) -> Option<&mut Process> {
        self.list.get_mut(pid as usize)
    }

    fn move_next(&mut self) {
        self.current = (self.current + 1) % self.list.len() as Pid;
    }

    fn mark_dead(&mut self, pid: Pid, exit_code: ExitCode) {
        if let Some(proc) = self.index(pid) {
            proc.state = ProcessState::Dead;
            proc.exit_code = exit_code;
        }
    }

    fn pop_current(&mut self) -> Option<Process> {
        if self.list.len() > 1 {
            Some(self.list.swap_remove(self.current as usize))
        } else {
            self.list.pop()
        }
    }
}

impl ProcessScheduler for FlatScheduler {
    fn add_process(&mut self, proc: Process) {
        //TODO: 检查当前进程列表,找到一个marked dead进程就将其替换,否则插入到末尾
        self.list.push(proc);
    }

    fn exit_process(&mut self, exit_code: ExitCode) {
        // 关闭时钟, 此时一般位于 ecall trap里,全局中断本来就关着吧?
        self.mark_dead(self.current, exit_code);
        self.switch_next();
    }

    fn switch_next(&mut self) -> Pid {
        let mut do_next = true;
        let mut do_clean = false;
        if let Some(current) = self.current() {
            match current.state {
                ProcessState::Idle => {
                    do_next = false;
                }
                ProcessState::Dead => {
                    do_clean = true;
                    do_next = false;
                }
                _ => (),
            }
        }
        if do_clean {
            if let Some(dead) = self.pop_current(){
                dead.cleanup();
                // TODO: notify children to call sys_call
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
            self.switch_next();
        } else {
            panic!("no process to be switched to");
        }
    }

    fn timer_tick(&mut self) {
        self.switch_next();
    }

    fn current(&mut self) -> Option<&mut Process> {
        self.list.get_mut(self.current as usize)
    }

    fn len(&self) -> usize {
        self.list.len()
    }
}
