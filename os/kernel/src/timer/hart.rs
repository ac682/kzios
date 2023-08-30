use crate::sbi;

use super::Timer;

const MS_PER_SEC: usize = 1000;

fn time() -> usize {
    // time refers to mtime, RustSBI will redirect and return the right value
    riscv::register::time::read()
}

pub struct HartTimer {
    frequency: usize,
    uptime: usize,
}

impl HartTimer {
    pub const fn new(freq: usize) -> Self {
        Self {
            frequency: freq,
            uptime: 0,
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

    fn schedule_next(&mut self, ms: usize) {
        let time = time();
        self.uptime = time * MS_PER_SEC / self.frequency;
        let interval = ms * self.frequency / MS_PER_SEC;
        let time = time + interval;
        self.set_timer(time);
    }

    fn put_off(&mut self) {
        // NOTE: 设置 time = usize::MAX 之后会 time++ 变成 0usize，直接触发，导致 put_off 失效
        self.set_timer(usize::MAX - 1);
    }
}
