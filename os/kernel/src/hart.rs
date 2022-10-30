use core::{cell::UnsafeCell, ptr::null_mut};

use alloc::{boxed::Box, sync::Arc, vec::Vec};
use erhino_shared::{
    call::{KernelCall, SystemCall},
    mem::PageNumber,
};
use num_traits::FromPrimitive;
use riscv::register::{
    mcause::{Exception, Interrupt, Mcause, Trap},
    mhartid,
};
use spin::Once;

use crate::{
    board::BoardInfo,
    external::_hart_num,
    mm::page::PageTableEntryFlag,
    println,
    proc::{
        sch::{self, flat::FlatScheduler, Scheduler},
        Process,
    },
    sync::{cell::UniProcessCell, optimistic::OptimisticLock, Lock},
    timer::hart::HartTimer,
    trap::TrapFrame,
};

// 内核陷入帧只会在第一个陷入时用到，之后大概率用不到，所以第一个陷入帧应该分配在一个垃圾堆(_memory_end - 4096)或者栈上
// 这么做是为了避免多核同时写入，但寻思了一下，根本不会去读，那多核写就写呗，写坏了也无所谓
// 那么！就这么决定了，这个内核陷入帧是只写的！也就是它是所谓的垃圾堆！
#[export_name = "_kernel_trap"]
static mut KERNEL_TRAP: TrapFrame = TrapFrame::new();

static mut HARTS: Vec<Hart> = Vec::new();

type SchedulerImpl = FlatScheduler<HartTimer>;

pub struct Hart {
    id: usize,
    timer: Arc<UniProcessCell<HartTimer>>,
    scheduler: SchedulerImpl,
    context: *mut TrapFrame,
}

impl Hart {
    pub fn new(hartid: usize, freq: usize) -> Self {
        let timer = Arc::new(UniProcessCell::new(HartTimer::new(hartid, freq)));
        
        Self {
            id: hartid,
            timer: timer.clone(),
            scheduler: SchedulerImpl::new(hartid, timer),
            context: unsafe { &mut KERNEL_TRAP as *mut TrapFrame },
        }
    }

    pub fn id(&self) -> usize {
        self.id
    }

    pub fn context(&self) -> &TrapFrame {
        unsafe { &*self.context }
    }

    pub fn handle_trap(&mut self, cause: Mcause) {
        let frame = unsafe { &mut *self.context };
        match cause.cause() {
            Trap::Interrupt(Interrupt::MachineTimer) => {
                self.timer.get_mut().tick();
                self.scheduler.tick();
            }
            Trap::Interrupt(Interrupt::MachineSoft) => {
                panic!("Machine Soft Interrupt at hart#{}", self.id);
                // its time to schedule process!
            }
            Trap::Exception(Exception::Breakpoint) => {
                // panic!("user breakpoint at hart#{}: frame=\n{}", self.id, frame);
                println!("{}", frame.x[10]);
                frame.pc += 4;
            }
            Trap::Exception(Exception::StoreFault) => {
                panic!("Store/AMO access fault hart#{}: frame=\n{}", self.id, frame);
            }
            Trap::Exception(Exception::MachineEnvCall) => {
                let call_id = frame.x[17];
                if let Some(call) = KernelCall::from_u64(call_id) {
                    match call {
                        KernelCall::EnterUserSpace => {
                            self.scheduler.begin();
                            frame.pc += 4
                        }
                    }
                } else {
                    panic!("unsupported kernel call: {}", call_id);
                }
            }
            Trap::Exception(Exception::UserEnvCall) => {
                let call_id = frame.x[17];
                if let Some(call) = SystemCall::from_u64(call_id) {
                    match call {
                        SystemCall::Extend => {
                            if let Some(process) = self.scheduler.current() {
                                let start = frame.x[10];
                                let end = start + frame.x[11];
                                let bits = frame.x[12] & 0b111;
                                let flags = if bits & 0b100 == 0b100 {
                                    PageTableEntryFlag::Executable
                                } else {
                                    PageTableEntryFlag::Valid
                                } | if bits & 0b010 == 0b010 {
                                    PageTableEntryFlag::Writeable
                                } else {
                                    PageTableEntryFlag::Valid
                                } | if bits & 0b001 == 0b001 {
                                    PageTableEntryFlag::Readable
                                } else {
                                    PageTableEntryFlag::Valid
                                };
                                process.memory.fill(
                                    (start >> 12) as PageNumber,
                                    ((end - start) >> 12) as PageNumber,
                                    PageTableEntryFlag::Valid | PageTableEntryFlag::User | flags,
                                );
                            }
                        },
                        SystemCall::Exit => {
                            self.scheduler.finish();
                        }
                        SystemCall::Yield => {
                            self.scheduler.tick();
                        }
                        _ => todo!("handle something"),
                    }
                    frame.pc += 4;
                } else {
                    println!("unsupported system call {}", call_id);
                    // kill the process or do something
                }
            }
            _ => panic!(
                "unknown trap cause at hart#{}: cause={:#x}, frame=\n{}",
                self.id,
                cause.bits(),
                frame
            ),
        }
        self.context = if let Some(proc) = self.scheduler.current() {
            &mut proc.trap as *mut TrapFrame
        } else {
            unsafe { &mut KERNEL_TRAP as *mut TrapFrame }
        }
    }
}

pub fn init(info: &BoardInfo) {
    unsafe {
        for i in 0..(_hart_num as usize) {
            HARTS.push(Hart::new(i, info.base_frequency))
        }
    }
}

pub fn my_hart() -> &'static mut Hart {
    let hartid = mhartid::read();
    of_hart(hartid)
}

// do not refer to other hart which has no locking state
pub fn of_hart(hartid: usize) -> &'static mut Hart {
    unsafe { &mut HARTS[hartid] }
}

pub fn add_flat_process(proc: Process) {
    SchedulerImpl::add(proc);
}
