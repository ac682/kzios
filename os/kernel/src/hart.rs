use core::{
    arch::asm,
    sync::atomic::{AtomicUsize, Ordering},
};

use alloc::{string::String, vec::Vec};
use erhino_shared::{
    call::{SystemCall, SystemCallError},
    mem::{Address, MemoryRegionAttribute},
    proc::ExitCode,
};

use crate::{
    external::{_awaken, _park, _switch},
    mm::{page::PAGE_BITS, ProcessAddressRegion, KERNEL_SATP},
    println, sbi,
    task::{
        proc::{Process, ProcessHealth, ProcessMemoryError},
        sched::{unfair::UnfairScheduler, ScheduleContext, Scheduler},
        thread::Thread,
    },
    timer::{hart::HartTimer, Timer},
    trap::TrapCause,
};

type TimerImpl = HartTimer;
type SchedulerImpl = UnfairScheduler;

static mut HARTS: Vec<Hart<TimerImpl, SchedulerImpl>> = Vec::new();
static IDLE_HARTS: AtomicUsize = AtomicUsize::new(0);

#[derive(Debug, PartialEq, Eq)]
pub enum HartStatus {
    Stopped,
    Suspended,
    Started,
}

pub struct Hart<T: Timer, S: Scheduler> {
    id: usize,
    timer: T,
    scheduler: S,
}

impl<T: Timer, S: Scheduler> Hart<T, S> {
    pub const fn new(hartid: usize, timer: T, scheduler: S) -> Self {
        Self {
            id: hartid,
            timer,
            scheduler,
        }
    }

    pub fn arranged_context(&mut self) -> (usize, Address) {
        if let Some((_, satp, trapframe)) = self.scheduler.context() {
            (satp, trapframe)
        } else {
            self.go_idle()
        }
    }

    pub fn send_ipi(&self) -> bool {
        sbi::send_ipi(1, self.id as isize).is_ok()
    }

    pub fn clear_ipi(&self) {
        // clear sip.SSIP => sip[1] = 0
        let mut sip: usize;
        unsafe {
            asm!("csrr {o}, sip",  o = out(reg) sip);
            asm!("csrw sip, {i}",i = in(reg) sip & !2);
        }
    }

    pub fn get_status(&self) -> Option<HartStatus> {
        if let Ok(ret) = sbi::hart_get_status(self.id) {
            match ret {
                0 | 2 | 6 => Some(HartStatus::Started),
                1 | 3 => Some(HartStatus::Stopped),
                4 => Some(HartStatus::Suspended),
                _ => None,
            }
        } else {
            None
        }
    }

    pub fn start(&self) -> bool {
        println!("#{} is awaking", self.id);
        sbi::hart_start(self.id, _awaken as usize, 0).is_ok()
    }

    pub fn suspend(&self) -> bool {
        sbi::hart_suspend(self.id, _awaken as usize, 0).is_ok()
    }

    pub fn stop(&self) -> bool {
        sbi::hart_stop(self.id).is_ok()
    }

    fn go_idle(&self) -> ! {
        IDLE_HARTS.fetch_or(1 << self.id, Ordering::Relaxed);
        println!("#{} enter idle", self.id);
        self.suspend();
        unsafe { _park() }
    }

    pub fn enter_user(&mut self) -> ! {
        self.scheduler.schedule();
        self.timer.schedule_next(self.scheduler.next_timeslice());
        if let Some((trampoline, satp, trapframe)) = self.scheduler.context() {
            unsafe { _switch(KERNEL_SATP, trampoline, satp, trapframe) }
        } else {
            self.go_idle()
        }
    }

    pub fn trap(&mut self, cause: TrapCause) {
        let mut schedule_request = false;
        // 同步 ecall 会直接操作并获得结果，PC+4
        // 异步 ecall 则只会将 task 状态设置为 Pending，PC 保持原样。调度器在解除其 Pending 状态成为 Fed 后重新加入调度，并触发 ecall，写入结果
        match cause {
            TrapCause::TimerInterrupt => {
                schedule_request = true;
            }
            TrapCause::Breakpoint => {
                self.scheduler.with_context(|ctx| {
                    ctx.trapframe().move_next_instruction();
                    println!(
                        "#{} Pid={} Tid={} requested a breakpoint",
                        self.id,
                        ctx.pid(),
                        ctx.tid()
                    );
                });
            }
            TrapCause::PageFault(address, _) => {
                if let Some(region) = self.scheduler.is_address_in(address) {
                    match region {
                        ProcessAddressRegion::Stack(_) => {
                            self.scheduler.with_context(|ctx| {
                                ctx.process()
                                    .fill(
                                        address >> PAGE_BITS,
                                        1,
                                        MemoryRegionAttribute::Write | MemoryRegionAttribute::Read,
                                        false,
                                    )
                                    .expect("fill stack failed, to be killed");
                            });
                        }
                        ProcessAddressRegion::TrapFrame(_) => {
                            self.scheduler.with_context(|ctx| {
                                ctx.process()
                                    .fill(
                                        address >> PAGE_BITS,
                                        1,
                                        MemoryRegionAttribute::Write | MemoryRegionAttribute::Read,
                                        true,
                                    )
                                    .expect("fill trapframe failed, to be killed");
                            });
                        }
                        _ => todo!("unexpected memory page fault at: {:#x}", address),
                    }
                } else {
                    unreachable!(
                        "previous process triggered page fault while it is not in current field"
                    )
                }
            }
            TrapCause::EnvironmentCall => self.scheduler.with_context(|ctx| {
                let trapframe = ctx.trapframe();
                trapframe.move_next_instruction();
                let process = ctx.process();
                let mut syscall = trapframe
                    .extract_syscall()
                    .expect("invalid sys call triggered");
                match syscall.call {
                    SystemCall::Debug => {
                        let address = syscall.arg0;
                        let length = syscall.arg1;
                        match process.read(address, length) {
                            Ok(buffer) => {
                                if let Ok(str) = String::from_utf8(buffer) {
                                    println!(
                                        "DBG#{} {}({}): {}",
                                        self.id,
                                        ctx.pid(),
                                        ctx.tid(),
                                        str
                                    );
                                    syscall.write_response(length)
                                } else {
                                    syscall.write_error(SystemCallError::IllegalArgument)
                                }
                            }
                            Err(e) => syscall.write_error(match e {
                                ProcessMemoryError::InaccessibleRegion => {
                                    SystemCallError::MemoryNotAccessible
                                }
                                _ => SystemCallError::Unknown,
                            }),
                        }
                    }
                    SystemCall::Exit => {
                        let code = syscall.arg0 as ExitCode;
                        process.health = ProcessHealth::Dead(code);
                        schedule_request = true;
                        syscall.write_response(0);
                    }
                    SystemCall::Extend => {
                        let bytes = syscall.arg0;
                        match process.extend(bytes) {
                            Ok(position) => {
                                syscall.write_response(position);
                            }
                            Err(err) => {
                                syscall.write_error(match err {
                                    ProcessMemoryError::OutOfMemory => SystemCallError::OutOfMemory,
                                    ProcessMemoryError::MisalignedAddress => {
                                        SystemCallError::MisalignedAddress
                                    }
                                    _ => SystemCallError::Unknown,
                                });
                            }
                        }
                    }
                    SystemCall::ThreadSpawn => {
                        let func_pointer = syscall.arg0 as Address;
                        let thread = Thread::new(func_pointer);
                        let tid = ctx.add_thread(thread);
                        syscall.write_response(tid as usize)
                    }
                    _ => unimplemented!("unimplemented syscall: {:?}", syscall.call),
                }
            }),
            _ => {
                todo!("unimplemented trap cause {:?}", cause)
            }
        }
        if schedule_request {
            self.scheduler.schedule();
            self.timer.schedule_next(self.scheduler.next_timeslice());
        }
    }
}

pub fn init(hart_num: usize, freq: &[usize]) {
    unsafe {
        for i in 0..hart_num {
            HARTS.push(Hart::new(
                i,
                TimerImpl::new(if i < freq.len() {
                    freq[i]
                } else {
                    freq[freq.len() - 1]
                }),
                SchedulerImpl::new(i),
            ));
        }
    }
}

pub fn send_ipi(hart_mask: usize) -> bool {
    if let Ok(_) = sbi::send_ipi(hart_mask, 0) {
        true
    } else {
        false
    }
}

pub fn start_all() {
    for i in unsafe { &HARTS } {
        if let Some(HartStatus::Stopped) = i.get_status() {
            i.start();
        }
    }
}

pub fn awake_idle() -> bool {
    let map = IDLE_HARTS.load(Ordering::Relaxed);
    send_ipi(map)
}

pub fn send_ipi_all() -> bool {
    if let Ok(_) = sbi::send_ipi(0, -1) {
        true
    } else {
        false
    }
}

pub fn get_hart(id: usize) -> &'static mut Hart<TimerImpl, SchedulerImpl> {
    unsafe {
        if id < HARTS.len() {
            &mut HARTS[id as usize]
        } else {
            panic!("reference to hart id {} is out of bound", id);
        }
    }
}

pub fn hartid() -> usize {
    let mut tp: usize;
    unsafe {
        asm!("mv {tmp}, tp", tmp = out(reg) tp);
    }
    tp
}

pub fn this_hart() -> &'static mut Hart<TimerImpl, SchedulerImpl> {
    get_hart(hartid())
}

pub fn add_process(proc: Process) {
    this_hart().scheduler.add(proc, None);
}
