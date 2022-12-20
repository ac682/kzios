use core::{cell::RefCell, mem::size_of, slice::from_raw_parts};

use alloc::{boxed::Box, rc::Rc, vec::Vec};
use erhino_shared::{
    call::{KernelCall, SystemCall},
    mem::{Address, PageNumber},
    proc::{ExitCode, Pid, ProcessInfo, ProcessPermission},
    service::Sid,
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
        sch::{smooth::SmoothScheduler, Scheduler},
        Process,
    },
    sync::hart::HartReadWriteLock,
    timer::{hart::HartTimer, Timer},
    trap::TrapFrame,
};

extern "Rust"{
    fn board_hart_awake();
}

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

type SchedulerImpl = SmoothScheduler<HartTimer>;

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
            scheduler: Box::new(SchedulerImpl::new(hartid, timer.clone())),
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
                let start = self.timer.borrow_mut().get_cycles();
                self.scheduler.tick();
                let end = self.timer.borrow_mut().get_cycles();
                let cost = end - start;
                let all = self.timer.borrow_mut().ms_to_cycles(50);
                println!("Scheduling cost {}/{}({}%) cycles", cost,all, 100 as f64 * cost as f64 / all as f64);
            }
            Trap::Interrupt(Interrupt::MachineSoft) => {
                peripheral::aclint().clear_msip(self.id);
                unsafe {
                    board_hart_awake();
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
                if let Some((current, _)) = self.scheduler.current() {
                    // panic!("breakpoint: memory={}\nframe={}", current.memory, frame);
                    println!("#{} Pid={} requested a breakpoint", self.id, current.pid);
                    frame.pc += 4;
                } else {
                    panic!("breakpoint: frame={}", frame);
                }
            }
            Trap::Exception(Exception::StoreFault) => {
                panic!("Store/AMO access fault hart#{}: frame=\n{}", self.id, frame);
            }
            Trap::Exception(Exception::StorePageFault) => {
                if let Some((process, _)) = self.scheduler.current() {
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
                            if let Some((current, _)) = self.scheduler.current() {
                                let start = frame.x[10] as usize;
                                let str = current.memory.read_cstr(start).unwrap();
                                print!(
                                    "\x1b[0;35m#{} Pid={}: {}\x1b[0;m",
                                    self.id, current.pid, str
                                );
                            }
                        }
                        SystemCall::Write => {
                            if let Some((current, _)) = self.scheduler.current() {
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
                            if let Some((process, _)) = self.scheduler.current() {
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
                                if let Ok(_perm_into) = FlagSet::<ProcessPermission>::new(perm as u8)
                                {
                                    // if let Some((current, thread)) = self.scheduler.current() {
                                    //     if let Ok(mut fork) = current.fork(perm_into) {
                                    //         fork.move_to_next_instruction();
                                    //         fork.trap.x[10] = 0;
                                    //         if fork.state == ProcessState::Running {
                                    //             fork.state = ProcessState::Ready;
                                    //         }
                                    //         let pid = self.scheduler.add(fork);
                                    //         ret = pid as i64;
                                    //     } else {
                                    //         ret = -2;
                                    //     }
                                    // }

                                    // fork 应该由 scheduler 处理
                                    todo!()
                                } else {
                                    ret = -3;
                                }
                            } else {
                                ret = -1;
                            }
                        }
                        SystemCall::Exit => {
                            let code = frame.x[10] as ExitCode;
                            self.scheduler.finish(code);
                        }
                        SystemCall::Yield => {
                            self.scheduler.tick();
                        }
                        SystemCall::Inspect => {
                            let address = frame.x[10] as Address;
                            let pid = frame.x[11] as Pid;
                            let name_address = frame.x[12] as Address;
                            let mut info: Option<ProcessInfo> = None;
                            if let Some(process) = self.scheduler.find(pid) {
                                info = Some(ProcessInfo {
                                    name: process.name.clone(),
                                    pid: process.pid,
                                    parent: process.parent,
                                    state: process.state.clone(),
                                    permissions: process.permissions.clone(),
                                });
                            } else {
                                ret = -1;
                            }
                            if ret != -1 {
                                if let Some((current, _)) = self.scheduler.current() {
                                    let info = info.unwrap();
                                    let mut name_buffer = info.name.as_bytes();
                                    let mut name_len = name_buffer.len();
                                    if name_len > 255 {
                                        name_buffer = &name_buffer[..255];
                                        name_len = 255;
                                    }
                                    let data = unsafe {
                                        from_raw_parts(
                                            &info as *const ProcessInfo as *const u8,
                                            size_of::<ProcessInfo>(),
                                        )
                                    };
                                    if current
                                        .memory
                                        .write(
                                            name_address,
                                            name_buffer,
                                            name_len + 1,
                                            PageTableEntryFlag::UserReadWrite
                                                | PageTableEntryFlag::Valid,
                                        )
                                        .is_err()
                                    {
                                        ret = -2;
                                    };
                                    if current
                                        .memory
                                        .write(
                                            address,
                                            data,
                                            data.len(),
                                            PageTableEntryFlag::UserReadWrite
                                                | PageTableEntryFlag::Valid,
                                        )
                                        .is_err()
                                    {
                                        ret = -2;
                                    };
                                }
                            }
                        }
                        SystemCall::InspectMyself => {
                            let address = frame.x[10] as Address;
                            let name_address = frame.x[12] as Address;
                            if let Some((current, _)) = self.scheduler.current() {
                                let info = ProcessInfo {
                                    name: current.name.clone(),
                                    pid: current.pid,
                                    parent: current.parent,
                                    state: current.state.clone(),
                                    permissions: current.permissions.clone(),
                                };
                                let mut name_buffer = info.name.as_bytes();
                                let mut name_len = name_buffer.len();
                                if name_len > 255 {
                                    name_buffer = &name_buffer[..255];
                                    name_len = 255;
                                }
                                let data = unsafe {
                                    from_raw_parts(
                                        &info as *const ProcessInfo as *const u8,
                                        size_of::<ProcessInfo>(),
                                    )
                                };
                                if current
                                    .memory
                                    .write(
                                        name_address,
                                        name_buffer,
                                        name_len + 1,
                                        PageTableEntryFlag::UserReadWrite
                                            | PageTableEntryFlag::Valid,
                                    )
                                    .is_err()
                                {
                                    ret = -2;
                                };
                                if current
                                    .memory
                                    .write(
                                        address,
                                        data,
                                        data.len(),
                                        PageTableEntryFlag::UserReadWrite
                                            | PageTableEntryFlag::Valid,
                                    )
                                    .is_err()
                                {
                                    ret = -2;
                                }
                            }
                        }
                        SystemCall::SignalReturn => {
                            // if let Some((current, thread)) = self.scheduler.current() {
                            //     current.leave_signal();
                            // }
                        }
                        SystemCall::SignalSet => {
                            // if let Some((current, thread)) = self.scheduler.current() {
                            //     let handler = frame.x[10];
                            //     let mask = frame.x[11];
                            //     current.set_signal_handler(handler as Address, mask);
                            // }
                        }
                        SystemCall::SignalSend => {
                            // let pid = frame.x[10] as Pid;
                            // let signal = frame.x[11] as Signal;
                            // if let Some(proc) = self.scheduler.find_mut(pid) {
                            //     proc.queue_signal(signal);
                            // }
                        }
                        SystemCall::ServiceRegister => {
                            let _sid = frame.x[10] as Sid;
                            if let Some((current, _)) = self.scheduler.current() {
                                let _pid = current.pid;
                                if current.has_permission(ProcessPermission::Service) {
                                    todo!("service_register sys call");
                                } else {
                                    ret = -7;
                                }
                            }
                        }
                        SystemCall::Map => {
                            if let Some((current, _)) = self.scheduler.current() {
                                if current.has_permission(ProcessPermission::Memory) {
                                    todo!("map sys call");
                                } else {
                                    ret = -7;
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
        self.context = if let Some((_proc, thread)) = self.scheduler.current() {
            //println!("Pid: {} Tid: {}", proc.pid, thread.tid);
            &mut thread.frame as *mut TrapFrame
        } else {
            unsafe { &mut KERNEL_TRAP as *mut TrapFrame }
        }
    }
}

pub fn init(info: &BoardInfo) {
    unsafe {
        // self awake
        board_hart_awake();
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
