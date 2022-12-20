use erhino_shared::mem::Address;
use riscv::register::time;

pub struct Aclint {
    mswi_address: usize,
    mtimer_address: usize,
    mtime_address: usize
}

impl Aclint {
    pub fn new(mswi_address: Address, mtimer_address: Address, mtime_address: Address) -> Self {
        Self {
            mswi_address,
            mtimer_address,
            mtime_address
        }
    }

    pub fn set_msip(&self, hartid: usize) {
        unsafe {
            (self.mswi_address as *mut u32)
                .add(hartid)
                .write_volatile(1)
        }
    }

    pub fn clear_msip(&self, hartid: usize) {
        unsafe {
            (self.mswi_address as *mut u32)
                .add(hartid)
                .write_volatile(0)
        }
    }

    pub fn get_time(&self) -> u64 {
        unsafe{
            (self.mtime_address as *const u64).read_volatile()
        }
    }

    pub fn set_timer(&self, hartid: usize, cycles: usize) {
        unsafe {
            (self.mtimer_address as *mut u64)
                .add(hartid)
                .write_volatile(self.get_time() + cycles as u64);
        }
    }

    pub fn cancel_timer(&self, hartid: usize) {
        unsafe {
            (self.mtimer_address as *mut u64)
                .add(hartid)
                .write_volatile(u64::MAX);
        }
    }
}
