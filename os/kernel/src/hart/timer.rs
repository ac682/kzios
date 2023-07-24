use crate::sbi;

// every tick event is based on TICKS rather than MS
const TICKS_PER_SEC: usize = 100;

pub struct HartTimer {
    hartid: usize,
    frequency: usize,
}

fn time() -> usize {
    // time refers to mtime, RustSBI will redirect and return the right value
    riscv::register::time::read()
}

impl HartTimer {
    pub const fn new(id: usize, freq: usize) -> Self {
        Self {
            hartid: id,
            frequency: freq,
        }
    }

    // in ticks
    pub fn uptime(&self) -> usize {
        time() * TICKS_PER_SEC / self.frequency
    }

    pub fn tick_freq(&self) -> usize {
        TICKS_PER_SEC
    }

    pub fn set_next_event(&self, ticks: usize) {
        let interval = ticks * self.frequency / TICKS_PER_SEC;
        if sbi::is_time_supported() {
            sbi::set_timer(interval);
        } else {
            sbi::legacy_set_timer(interval);
        }
    }
}
