use crate::{board, sbi};

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
        if board::this_board()
            .see()
            .is_extension_supported(sbi::SbiExtension::Time)
        {
            sbi::set_timer(interval).expect("sbi timer system broken");
        } else {
            sbi::legacy_set_timer(interval);
        }
    }
}
