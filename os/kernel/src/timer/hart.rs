use crate::{debug, hart, sbi};

use super::Timer;

// every tick event is based on TICKS rather than MS
const TICKS_PER_SEC: usize = 100;

fn time() -> usize {
    // time refers to mtime, RustSBI will redirect and return the right value
    riscv::register::time::read()
}

pub struct HartTimer {
    frequency: usize,
    uptime: usize,
    last_ticks: usize,
    last_count: usize,
}

impl HartTimer {
    pub const fn new(freq: usize) -> Self {
        Self {
            frequency: freq,
            uptime: 0,
            last_ticks: 0,
            last_count: 0,
        }
    }

    fn set_timer(&self, cycle: usize) {
        if sbi::is_time_supported() {
            sbi::set_timer(cycle).expect("sbi timer system broken");
        } else {
            sbi::legacy_set_timer(cycle);
        }
    }
}

impl Timer for HartTimer {
    fn uptime(&self) -> usize {
        self.uptime
    }

    fn tick_freq(&self) -> usize {
        TICKS_PER_SEC
    }

    fn schedule_next(&mut self, ticks: usize) {
        if self.last_count > 1000 {
            self.uptime = time() * TICKS_PER_SEC / self.frequency;
        } else {
            self.uptime += ticks;
            self.last_count += 1;
        }
        self.last_ticks = ticks;
        let interval = ticks * self.frequency / TICKS_PER_SEC;
        let time = time() + interval;
        self.set_timer(time);
    }

    fn put_off(&mut self) {
        // NOTE: 设置 time = usize::MAX 之后会 time++ 变成 0usize，直接触发，导致 put_off 失效
        self.set_timer(usize::MAX - 1);
    }
}
