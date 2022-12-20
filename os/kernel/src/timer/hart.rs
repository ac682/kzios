use riscv::register::mie;

use crate::peripheral;

use super::Timer;

pub struct HartTimer {
    hartid: usize,
    freq: usize,
    // 单位为秒
    uptime: usize,
    last_cycles: usize,
    total_cycles: usize,
}

impl HartTimer {
    pub fn new(hartid: usize, freq: usize) -> Self {
        peripheral::aclint().cancel_timer(hartid);
        Self {
            hartid,
            freq,
            uptime: 0,
            last_cycles: 0,
            total_cycles: 0,
        }
    }

    pub fn tick(&mut self) {
        self.total_cycles += self.last_cycles;
        if self.total_cycles > self.freq {
            self.total_cycles -= self.freq;
            self.uptime += 1;
        }
    }
}

impl Timer for HartTimer {
    fn get_uptime(&self) -> usize {
        self.uptime
    }

    fn set_timer(&mut self, cycles: usize) {
        peripheral::aclint().set_timer(self.hartid, cycles);
        unsafe { mie::set_mtimer() };
        self.last_cycles = cycles;
    }

    fn get_cycles(&self) -> usize {
        peripheral::aclint().get_time() as usize
    }

    fn ms_to_cycles(&self, time: usize) -> usize {
        self.freq * time / 1000
    }

    fn cycles_to_ms(&self, cycles: usize) -> usize {
        cycles * 1000 / self.freq
    }
}
