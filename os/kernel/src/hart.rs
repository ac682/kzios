use core::cell::RefCell;

use alloc::{boxed::Box, rc::Rc, vec::Vec};
use erhino_shared::{
    call::{KernelCall, SystemCall},
    mem::{Address, PageNumber},
    proc::{Pid, ProcessPermission, ProcessState, Signal},
};
use flagset::FlagSet;
use num_traits::FromPrimitive;
use riscv::register::{
    mcause::{Exception, Interrupt, Mcause, Trap},
    mhartid,
    mstatus::{self, MPP},
};

use crate::{
    board::BoardInfo,
    external::_hart_num,
    mm::page::PageTableEntryFlag,
    peripheral, print, println,
    proc::{
        sch::{flat::FlatScheduler, Scheduler},
        Process,
    },
    sync::hart::HartReadWriteLock,
    timer::hart::HartTimer,
    trap::TrapFrame,
};

// 内核陷入帧只会在第一个陷入时用到，之后大概率用不到，所以第一个陷入帧应该分配在一个垃圾堆(_memory_end - 4096)或者栈上
// 这么做是为了避免多核同时写入，但寻思了一下，根本不会去读，那多核写就写呗，写坏了也无所谓
// 那么！就这么决定了，这个内核陷入帧是只写的！也就是它是所谓的垃圾堆！
#[export_name = "_kernel_trap"]
static mut KERNEL_TRAP: TrapFrame = TrapFrame::new();

// 不同 Hart 实例要有不同的 Scheduler 类型。
// 以此达成"hart#0 执行用户程序 hart#1 执行实时进程"的目的
static mut HARTS: Vec<Hart> = Vec::new();

static mut SIZE: usize = 0usize;
static mut SIZE_LOCK: HartReadWriteLock = HartReadWriteLock::new();

type SchedulerImpl = FlatScheduler<HartTimer>;

pub struct Hart {
    id: usize,
    timer: Rc<RefCell<HartTimer>>,
    scheduler: Box<dyn Scheduler>,
    context: *mut TrapFrame,
}

impl Hart {
    pub fn new(hartid: usize, freq: usize) -> Self {
        let timer = Rc::new(RefCell::new(HartTimer::new(hartid, freq)));

        Self {
            id: hartid,
            timer: timer.clone(),
            scheduler: Box::new(SchedulerImpl::new(hartid, timer)),
            context: unsafe { &mut KERNEL_TRAP as *mut TrapFrame },
        }
    }

    pub fn id(&self) -> usize {
        self.id
    }

    pub fn context(&self) -> &TrapFrame {
        unsafe { &*self.context }
    }

    pub fn handle_trap_from_user(&mut self, cause: Mcause, val: usize) {
        let frame = unsafe { &mut *self.context };
        match cause.cause() {
            Trap::Interrupt(Interrupt::MachineTimer) => {
                self.timer.borrow_mut().tick();
                self.scheduler.tick();
            }
            Trap::Interrupt(Interrupt::MachineSoft) => {
                peripheral::aclint().clear_msip(self.id);
                unsafe {
                    mstatus::set_mpp(MPP::User);
                }
                self.scheduler.begin();
                frame.pc += 4;
            }
            Trap::Exception(Exception::MachineEnvCall) => {
                let call_id = frame.x[17];
                if let Some(call) = KernelCall::from_u64(call_id) {
                    match call {
                        KernelCall::EnterUserSpace => {
                            unsafe {
                                mstatus::set_mpp(MPP::User);
                            }
                            self.scheduler.begin();
                            frame.pc += 4;
                        }
                    }
                } else {
                    panic!("unsupported kernel call: {}", call_id);
                }
            }
            Trap::Exception(Exception::Breakpoint) => {
                panic!("breakpoint: frame={}", frame);
            }
            Trap::Exception(Exception::StoreFault) => {
                panic!("Store/AMO access fault hart#{}: frame=\n{}", self.id, frame);
            }
            Trap::Exception(Exception::StorePageFault) => {
                if let Some(process) = self.scheduler.current() {
                    if let Ok(success) = process
                        .memory
                        .handle_store_page_fault(val, PageTableEntryFlag::UserReadWrite)
                    {
                        if !success {
                            panic!(
                                "the memory program {}({}) accessed is not writeable: {:#x}",
                                process.name, process.pid, val
                            );
                        }
                    }
                } else {
                    panic!("ran out of memory");
                }
            }
            Trap::Exception(Exception::UserEnvCall) => {
                let call_id = frame.x[17];
                let mut ret = 0i64;
                if let Some(call) = SystemCall::from_u64(call_id) {
                    match call {
                        SystemCall::Debug => {
                            if let Some(current) = self.scheduler.current() {
                                let str_start = frame.x[10];
                                let str_len = frame.x[11];
                                let mut bytes = Vec::<u8>::new();
                                for _ in 0..str_len {
                                    bytes.push(0);
                                }
                                current
                                    .memory
                                    .read(str_start as Address, &mut bytes, str_len as usize)
                                    .unwrap();
                                let str = core::str::from_utf8(&bytes).unwrap();
                                print!("\x1b[0;35m{}\x1b[0;m", str);
                            }
                        }
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
                                process
                                    .memory
                                    .fill(
                                        (start >> 12) as PageNumber,
                                        ((end - start) >> 12) as PageNumber,
                                        PageTableEntryFlag::Valid
                                            | PageTableEntryFlag::User
                                            | flags,
                                    )
                                    .unwrap();
                            }
                        }
                        SystemCall::Fork => {
                            let perm = frame.x[11];
                            if perm <= u8::MAX as u64 {
                                if let Ok(perm_into) = FlagSet::<ProcessPermission>::new(perm as u8)
                                {
                                    if let Some(current) = self.scheduler.current() {
                                        if let Ok(mut fork) = current.fork(perm_into) {
                                            fork.move_to_next_instruction();
                                            fork.trap.x[10] = 0;
                                            if fork.state == ProcessState::Running {
                                                fork.state = ProcessState::Ready;
                                            }
                                            let pid = self.scheduler.add(fork);
                                            ret = pid as i64;
                                        } else {
                                            ret = -2;
                                        }
                                    }
                                } else {
                                    ret = -3;
                                }
                            } else {
                                ret = -1;
                            }
                        }
                        SystemCall::Exit => {
                            self.scheduler.finish();
                        }
                        SystemCall::Yield => {
                            self.scheduler.tick();
                        }
                        SystemCall::SignalReturn => {
                            if let Some(current) = self.scheduler.current() {
                                current.leave_signal();
                            }
                        }
                        SystemCall::SignalSet => {
                            if let Some(current) = self.scheduler.current() {
                                let handler = frame.x[10];
                                let mask = frame.x[11];
                                current.set_signal_handler(handler as Address, mask);
                            }
                        }
                        SystemCall::SignalSend => {
                            let pid = frame.x[10] as Pid;
                            let signal = frame.x[11] as Signal;
                            if let Some(proc) = self.scheduler.find_mut(pid) {
                                proc.queue_signal(signal);
                            }
                        }
                        SystemCall::Map => {
                            if let Some(current) = self.scheduler.current() {
                                if current.has_permission(ProcessPermission::Memory) {
                                    todo!("map sys call");
                                } else {
                                    ret = -1;
                                }
                            }
                        }
                        _ => todo!("handle something"),
                    }
                    frame.x[10] = ret as u64;
                    frame.pc += 4;
                } else {
                    println!("unsupported system call {}", call_id);
                    // kill the process or do something
                }
            }
            _ => panic!(
                "unknown trap cause at hart#{}: cause={:#x}, tval={:#x}, frame=\n{}",
                self.id,
                cause.bits(),
                val,
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
    my_hart().scheduler.add(proc);
}
