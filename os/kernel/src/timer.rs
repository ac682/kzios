use alloc::vec::Vec;
use riscv::register::mie;

use crate::{
    board::BoardInfo,
    external::_hart_num,
    peripheral::{self, aclint::Aclint},
    sync::{hart::HartReadWriteLock, Lock, ReadWriteLock},
};

static mut TIMER_DATA: HartReadWriteLock<TimerData> = HartReadWriteLock::empty();

struct TimerData {
    freq: usize,
    // 单位为秒
    uptime: usize,
    last_cycles: usize,
    total_cycles: usize,
    handlers: Vec<fn()>,
}

pub fn init(info: &BoardInfo) {
    unsafe {
        let mut vec = Vec::<fn()>::new();
        for i in 0..(_hart_num as usize) {
            vec.push(|| {});
            peripheral::aclint().cancel_timer(i);
        }
        mie::set_mtimer();
        TIMER_DATA.put(TimerData {
            freq: info.base_frequency,
            uptime: 0,
            last_cycles: 0,
            total_cycles: 0,
            handlers: vec,
        });
    }
}

pub fn tick(hartid: usize) {
    // 计算时间更新 uptime
    let mut data = unsafe { TIMER_DATA.lock_mut() };
    data.total_cycles += data.last_cycles;
    if data.total_cycles > data.freq{
        data.total_cycles -= data.freq;
        data.uptime += 1;
    }
    data.handlers[hartid]();
}

pub fn get_uptime() -> usize {
    unsafe { TIMER_DATA.lock().uptime }
}

pub fn get_time() -> usize{
    peripheral::aclint().get_time() as usize
}

pub fn set_timer(hartid: usize, cycles: usize, handler: fn()) {
    peripheral::aclint().set_timer(hartid, cycles);
    let mut data = unsafe { TIMER_DATA.lock_mut() };
    data.last_cycles = cycles;
    data.handlers[hartid] = handler;
}

pub fn time_to_cycles(ms: usize) -> usize {
    let data = unsafe { &*TIMER_DATA.access()};
    data.freq * ms / 1000
}
