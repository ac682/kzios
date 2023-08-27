use core::{
    arch::asm,
    ops::Index,
    sync::atomic::{AtomicUsize, Ordering},
};

use alloc::{string::String, vec::Vec};
use erhino_shared::{
    call::{SystemCall, SystemCallError},
    mem::{Address, MemoryRegionAttribute},
    proc::{ExitCode, Pid, SignalMap},
    sync::DataLock,
};

use crate::{
    debug,
    external::{_awaken, _park, _switch},
    mm::{
        frame,
        page::{PAGE_BITS, PAGE_SIZE},
        ProcessAddressRegion, KERNEL_SATP,
    },
    println,
    rng::RandomGenerator,
    sbi,
    sync::spin::SpinLock,
    task::{
        ipc::tunnel::Tunnel,
        proc::{Process, ProcessHealth, ProcessMemoryError},
        sched::{ScheduleContext, Scheduler},
        thread::Thread,
    },
    timer::Timer,
    trap::TrapCause,
};

use super::{enter_user, send_ipi, this_hart, HartKind, HartStatus};

static IDLE_HARTS: AtomicUsize = AtomicUsize::new(0);
static TUNNELS: DataLock<Vec<Tunnel>, SpinLock> = DataLock::new(Vec::new(), SpinLock::new());

pub struct ApplicationHart<T, S, R> {
    id: usize,
    timer: T,
    scheduler: S,
    random: R,
}

impl<T: Timer, S: Scheduler, R: RandomGenerator> ApplicationHart<T, S, R> {
    pub const fn new(hartid: usize, timer: T, scheduler: S, random: R) -> Self {
        Self {
            id: hartid,
            timer,
            scheduler,
            random,
        }
    }

    pub fn id(&self) -> usize {
        self.id
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
        sbi::hart_start(self.id, _awaken as usize, enter_user as usize).is_ok()
    }

    pub fn suspend(&self) -> bool {
        sbi::hart_suspend(0, _awaken as usize, 0).is_ok()
    }

    pub fn stop(&self) -> bool {
        sbi::hart_stop().is_ok()
    }

    fn go_idle(&mut self) -> ! {
        debug!("#{} idle", self.id());
        self.timer.put_off();
        IDLE_HARTS.fetch_or(1 << self.id, Ordering::Relaxed);
        self.suspend();
        unsafe { _park() }
    }

    pub fn uptime(&self) -> usize {
        self.timer.uptime() / self.timer.tick_freq()
    }

    pub fn go_awaken(&mut self) -> ! {
        debug!("#{} awaken", self.id());
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
                                let str = unsafe { String::from_utf8_unchecked(buffer) };
                                println!("\x1b[0;34mUSER\x1b[0m {}({}): {}", ctx.pid(), ctx.tid(), str);
                                syscall.write_response(length)
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
                    SystemCall::TunnelBuild => {
                        if let Some(frame) = frame::borrow(1) {
                            let mut rng = self.random.next();
                            let mut tunnels = TUNNELS.lock();
                            while tunnels.iter().any(|t| t.key() == rng) {
                                rng = self.random.next();
                            }
                            let tunnel = Tunnel::new(rng, ctx.pid(), frame);
                            tunnels.push(tunnel);
                            syscall.write_response(rng);
                        } else {
                            syscall.write_error(SystemCallError::OutOfMemory);
                        }
                    }
                    SystemCall::TunnelLink => {
                        let key = syscall.arg0;
                        let mut tunnels = TUNNELS.lock();
                        let mut found = false;
                        for tunnel in tunnels.iter_mut() {
                            if tunnel.key() == key {
                                let addr = {
                                    let heap = ctx.process().break_point();
                                    let stack = ctx.process().stack_point();
                                    let mid =
                                        (((stack - heap) / 2) + heap) & (!0usize - (PAGE_SIZE - 1));
                                    mid - ctx.process().usage.tunnel * PAGE_SIZE
                                };
                                let occupied = !tunnel.link(ctx.pid(), addr >> PAGE_BITS);
                                if !occupied {
                                    if ctx
                                        .process()
                                        .map(
                                            addr >> PAGE_BITS,
                                            tunnel.page_number(),
                                            1,
                                            MemoryRegionAttribute::Read
                                                | MemoryRegionAttribute::Write,
                                            false,
                                        )
                                        .is_ok()
                                    {
                                        ctx.process().usage.tunnel += 1;
                                        syscall.write_response(addr);
                                    } else {
                                        tunnel.unlink(ctx.pid());
                                        syscall.write_error(SystemCallError::MemoryNotAccessible);
                                    }
                                } else {
                                    syscall.write_error(SystemCallError::ObjectNotAccessible);
                                }
                                found = true;
                                break;
                            }
                        }
                        if !found {
                            syscall.write_error(SystemCallError::ObjectNotFound);
                        }
                    }
                    SystemCall::TunnelDispose => {
                        let key = syscall.arg0;
                        let mut tunnels = TUNNELS.lock();
                        let mut found = false;
                        let mut delete = false;
                        let mut delete_index = 0usize;
                        for (index, tunnel) in tunnels.iter_mut().enumerate() {
                            if tunnel.key() == key {
                                if let Some((d, _number)) = tunnel.unlink(ctx.pid()) {
                                    delete_index = index;
                                    delete = d;
                                    // TODO: ctx.process().free(n, 1)
                                    // TODO: 进程退出的时候检查所有 owner 为 pid 的对象，确定 second == None && first == None | pid 后删除（可省略 unlink）
                                    ctx.process().usage.tunnel -= 1;
                                    syscall.write_response(0);
                                } else {
                                    syscall.write_error(SystemCallError::ObjectNotAccessible);
                                }
                                found = true;
                                break;
                            }
                        }
                        if found {
                            if delete {
                                tunnels.swap_remove(delete_index);
                            }
                        } else {
                            syscall.write_error(SystemCallError::ObjectNotFound);
                        }
                    }
                    SystemCall::SignalSet => {
                        let mask = syscall.arg0;
                        let handler = syscall.arg1;
                        ctx.process().signal.set_handler(mask as SignalMap, handler);
                        syscall.write_response(0);
                    }
                    SystemCall::SignalSend => {
                        let pid = syscall.arg0 as Pid;
                        let signal = syscall.arg1 as SignalMap;
                        if !self.scheduler.find(pid, |target| {
                            if target.signal.is_accepted(signal) {
                                target.signal.enqueue(signal);
                                syscall.write_response(1);
                            } else {
                                syscall.write_response(0);
                            }
                        }) {
                            syscall.write_error(SystemCallError::ObjectNotFound);
                        }
                    }
                    SystemCall::SignalReturn => {
                        let proc = ctx.process();
                        if proc.signal.is_handling() {
                            proc.signal.complete();
                            syscall.write_response(0);
                        } else {
                            syscall.write_error(SystemCallError::FunctionNotAvailable);
                        }
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

pub fn awake_idle() -> bool {
    let map = IDLE_HARTS.load(Ordering::Relaxed);
    send_ipi(map)
}

pub fn add_process(proc: Process) {
    if let HartKind::Application(hart) = this_hart() {
        hart.scheduler.add(proc, None);
    }
}
