use core::{
    arch::asm,
    sync::atomic::{AtomicUsize, Ordering},
};

use alloc::{string::String, vec::Vec};
use erhino_shared::{
    call::{SystemCall, SystemCallError},
    mem::{Address, MemoryRegionAttribute},
    proc::{ExitCode, Pid, ProcessPermission, SignalMap},
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
        proc::{Process, ProcessHealth, ProcessMemoryError, ProcessTunnelError},
        sched::{ScheduleContext, Scheduler},
        thread::Thread,
    },
    trap::TrapCause,
};

use super::{enter_user, send_ipi, this_hart, HartKind, HartStatus};

static IDLE_HARTS: AtomicUsize = AtomicUsize::new(0);
static TUNNELS: DataLock<Vec<Tunnel>, SpinLock> = DataLock::new(Vec::new(), SpinLock::new());

pub struct ApplicationHart<S, R> {
    id: usize,
    scheduler: S,
    random: R,
}

impl<S: Scheduler, R: RandomGenerator> ApplicationHart<S, R> {
    pub const fn new(hartid: usize, scheduler: S, random: R) -> Self {
        Self {
            id: hartid,
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

    pub fn _send_ipi(&self) -> bool {
        sbi::send_ipi(1, self.id as isize).is_ok()
    }

    pub fn _clear_ipi(&self) {
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
        sbi::hart_suspend(0, _awaken as usize, enter_user as usize).is_ok()
    }

    pub fn _stop(&self) -> bool {
        sbi::hart_stop().is_ok()
    }

    fn go_idle(&mut self) -> ! {
        debug!("#{} idle", self.id());
        self.scheduler.cancel();
        IDLE_HARTS.fetch_or(1 << self.id, Ordering::Relaxed);
        self.suspend();
        unsafe { _park() }
    }

    pub fn go_awaken(&mut self) -> ! {
        debug!("#{} awaken", self.id());
        self.scheduler.schedule();
        if let Some((trampoline, satp, trapframe)) = self.scheduler.context() {
            unsafe { _switch(KERNEL_SATP, trampoline, satp, trapframe) }
        } else {
            self.go_idle()
        }
    }

    fn _handle_remote_call() {}

    fn handle_system_call(
        context: &mut S::Context,
        call: SystemCall,
        arg0: usize,
        arg1: usize,
        _arg2: usize,
        random: &mut R
    ) -> Result<usize, SystemCallError> {
        let process = context.process();
        match call {
            SystemCall::Debug => {
                let address = arg0;
                let length = arg1;
                match process.read(address, length) {
                    Ok(buffer) => {
                        let str = unsafe { String::from_utf8_unchecked(buffer) };
                        println!(
                            "\x1b[0;34mUSER\x1b[0m {}({}): {}",
                            context.pid(),
                            context.tid(),
                            str
                        );
                        Ok(length)
                    }
                    Err(e) => Err(match e {
                        ProcessMemoryError::InaccessibleRegion => {
                            SystemCallError::MemoryNotAccessible
                        }
                        _ => SystemCallError::Unknown,
                    }),
                }
            }
            SystemCall::ClaimTheUnlimitedPower => {
                if process.has_permission(ProcessPermission::LimitedPower) {
                    todo!("map the pages and write address & length to response")
                } else {
                    Err(SystemCallError::PermissionDenied)
                }
            }
            SystemCall::Exit => {
                let code = arg0 as ExitCode;
                process.health = ProcessHealth::Dead(code);
                context.schedule();
                Ok(0)
            }
            SystemCall::Extend => {
                let bytes = arg0;
                process.extend(bytes).map_err(|err| match err {
                    ProcessMemoryError::OutOfMemory => SystemCallError::OutOfMemory,
                    ProcessMemoryError::MisalignedAddress => SystemCallError::MisalignedAddress,
                    _ => SystemCallError::Unknown,
                })
            }
            SystemCall::ThreadSpawn => {
                let func_pointer = arg0 as Address;
                let thread = Thread::new(func_pointer);
                let tid = context.add_thread(thread);
                Ok(tid as usize)
            }
            SystemCall::TunnelBuild => {
                if let Some(frame) = frame::borrow(1) {
                    let mut rng = random.next();
                    let mut tunnels = TUNNELS.lock();
                    while tunnels.iter().any(|t| t.key() == rng) {
                        rng = random.next();
                    }
                    let tunnel = Tunnel::new(rng, context.pid(), frame);
                    tunnels.push(tunnel);
                    Ok(rng)
                } else {
                    Err(SystemCallError::OutOfMemory)
                }
            }
            SystemCall::TunnelLink => {
                let key = arg0;
                let mut tunnels = TUNNELS.lock();
                let mut found: Option<&mut Tunnel> = None;
                for tunnel in tunnels.iter_mut() {
                    if tunnel.key() == key {
                        found = Some(tunnel);
                        break;
                    }
                }
                if let Some(tunnel) = found {
                    match process.tunnel_insert(key) {
                        Ok(slot) => {
                            let addr = process.tunnel_point() + PAGE_SIZE * slot;
                            let occupied = !tunnel.link(context.pid(), addr >> PAGE_BITS);
                            if !occupied {
                                if process
                                    .map(
                                        addr >> PAGE_BITS,
                                        tunnel.page_number(),
                                        1,
                                        MemoryRegionAttribute::Read | MemoryRegionAttribute::Write,
                                        false,
                                    )
                                    .is_ok()
                                {
                                    Ok(addr)
                                } else {
                                    tunnel.unlink(context.pid());
                                    Err(SystemCallError::MemoryNotAccessible)
                                }
                            } else {
                                Err(SystemCallError::ObjectNotAccessible)
                            }
                        }
                        Err(err) => match err {
                            ProcessTunnelError::ReachLimit => Err(SystemCallError::ReachLimit),
                        },
                    }
                } else {
                    Err(SystemCallError::ObjectNotFound)
                }
            }
            SystemCall::TunnelDispose => {
                let key = arg0;
                let mut tunnels = TUNNELS.lock();
                let mut found: Option<&mut Tunnel> = None;
                let mut delete_index = 0usize;
                for (index, tunnel) in tunnels.iter_mut().enumerate() {
                    if tunnel.key() == key {
                        delete_index = index;
                        found = Some(tunnel);
                        break;
                    }
                }
                if let Some(tunnel) = found {
                    if let Some((delete, _number)) = tunnel.unlink(context.pid()) {
                        if delete {
                            tunnels.swap_remove(delete_index);
                        }
                        // TODO: ctx.process().free(n, 1)
                        // 进程退出的时候检查所有 owner 为 pid 的对象，确定 second == None && first == None | pid 后删除（可省略 unlink）
                        // 进程退出的时候直接按照 tunnels 表删就可以，不需要逐个检查
                        process.tunnel_eject(key);
                        Ok(0)
                    } else {
                        Err(SystemCallError::ObjectNotAccessible)
                    }
                } else {
                    Err(SystemCallError::ObjectNotFound)
                }
            }
            SystemCall::SignalSet => {
                let mask = arg0;
                let handler = arg1;
                process.signal.set_handler(mask as SignalMap, handler);
                Ok(0)
            }
            SystemCall::SignalSend => {
                let pid = arg0 as Pid;
                let signal = arg1 as SignalMap;
                let mut accepted = false;
                if context.find(pid, |target| {
                    if target.signal.is_accepted(signal) {
                        target.signal.enqueue(signal);
                        accepted = true;
                    }
                }) {
                    Ok(if accepted { 1 } else { 0 })
                } else {
                    Err(SystemCallError::ObjectNotFound)
                }
            }
            SystemCall::SignalReturn => {
                if process.signal.is_handling() {
                    process.signal.complete();
                    Ok(0)
                } else {
                    Err(SystemCallError::FunctionNotAvailable)
                }
            }
            _ => unimplemented!("unimplemented syscall: {:?}", call),
        }
    }

    pub fn trap(&mut self, cause: TrapCause) {
        // 同步 ecall 会直接操作并获得结果，PC+4
        // 异步 ecall 则只会将 task 状态设置为 Pending，PC 保持原样。调度器在解除其 Pending 状态成为 Fed 后重新加入调度，并触发 ecall，写入结果
        match cause {
            TrapCause::TimerInterrupt => {
                self.scheduler.schedule();
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
                // 只有同步调用才会前进下一个指令
                // `handle_system_call` 返回 Ok(SystemCallProcedureResult)，包含 Finished(usize) 和 Pending()，后者会切换到其他进程
                trapframe.move_next_instruction();
                let mut syscall = trapframe
                    .extract_syscall()
                    .expect("invalid sys call triggered");
                match Self::handle_system_call(
                    ctx,
                    syscall.call,
                    syscall.arg0,
                    syscall.arg1,
                    syscall.arg2,
                    &mut self.random
                ) {
                    Ok(res) => syscall.write_response(res),
                    Err(err) => syscall.write_error(err),
                }
            }),
            _ => {
                todo!("unimplemented trap cause {:?}", cause)
            }
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
