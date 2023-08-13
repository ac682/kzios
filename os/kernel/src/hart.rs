use core::arch::asm;

use alloc::{string::String, vec::Vec};
use erhino_shared::{
    call::{SystemCall, SystemCallError},
    mem::{Address, MemoryRegionAttribute},
};

use crate::{
    external::{_hart_num, _switch},
    mm::{page::PAGE_BITS, ProcessAddressRegion},
    println, sbi,
    task::{
        proc::{Process, ProcessMemoryError},
        sched::{unfair::UnfairScheduler, Scheduler},
    },
    timer::{hart::HartTimer, Timer},
    trap::TrapCause,
};

type TimerImpl = HartTimer;
type SchedulerImpl = UnfairScheduler;

static mut HARTS: Vec<Hart<TimerImpl, SchedulerImpl>> = Vec::new();

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

    pub fn id(&self) -> usize {
        self.id
    }

    pub fn arranged_context(&self) -> (usize, Address) {
        let (_, satp, trapframe) = self.scheduler.context();
        (satp, trapframe)
    }

    pub fn send_ipi(&self) -> bool {
        if let Ok(_) = sbi::send_ipi(1, self.id as isize) {
            true
        } else {
            false
        }
    }

    pub fn clear_ipi(&self) {
        // clear sip.SSIP => sip[1] = 0
        let mut sip = 0usize;
        unsafe { asm!("csrr {o}, sip", "csrw sip, {i}", o = out(reg) sip, i = in(reg) sip & !2) }
    }

    pub fn enter_user(&mut self) -> ! {
        self.scheduler.schedule();
        let (trampoline, satp, trapframe) = self.scheduler.context();
        unsafe { _switch(trampoline, satp, trapframe) }
    }

    pub fn trap(&mut self, cause: TrapCause) {
        match cause {
            TrapCause::TimerInterrupt => {
                self.scheduler.schedule();
                self.timer.schedule_next(self.scheduler.next_timeslice());
            }
            TrapCause::Breakpoint => {
                self.scheduler.with_context(|p, t, f| {
                    f.move_next_instruction();
                    println!(
                        "#{} Pn={} Tn={} requested a breakpoint",
                        self.id, p.name, t.name
                    );
                });
            }
            TrapCause::PageFault(address, _) => {
                let region = self.scheduler.is_address_in(address);
                match region {
                    ProcessAddressRegion::Stack(_) => {
                        self.scheduler.with_context(|p, _, _| {
                            p.fill(
                                address >> PAGE_BITS,
                                1,
                                MemoryRegionAttribute::Write | MemoryRegionAttribute::Read,
                                false,
                            )
                            .expect("fill stack failed, to be killed");
                        });
                    }
                    ProcessAddressRegion::TrapFrame(_) => {
                        self.scheduler.with_context(|p, _, _| {
                            p.fill(
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
            }
            TrapCause::EnvironmentCall => self.scheduler.with_context(|p, t, f| {
                f.move_next_instruction();
                let mut syscall = f.extract_syscall().expect("invalid sys call triggered");
                match syscall.call {
                    SystemCall::Debug => {
                        let address = syscall.arg0;
                        let length = syscall.arg1;
                        match p.read(address, length) {
                            Ok(buffer) => {
                                if let Ok(str) = String::from_utf8(buffer) {
                                    println!("DBG {}/{}: {}", p.name, t.name, str);
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
                    SystemCall::Extend => {
                        let bytes = syscall.arg0;
                        match p.extend(bytes) {
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
                    _ => unimplemented!("unimplemented syscall: {:?}", syscall.call),
                }
            }),
            _ => {
                todo!("unimplemented trap cause {:?}", cause)
            }
        }
    }
}

pub fn init(freq: &[usize]) {
    unsafe {
        for i in 0..(_hart_num as usize) {
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
    println!("kernel process added: {}", proc.name);
    this_hart().scheduler.add(proc, None);
}
