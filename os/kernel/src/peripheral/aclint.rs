use crate::println;

pub struct Aclint {
    mswi_address: usize,
    mtimer_address: usize,
}

impl Aclint {
    pub fn new(mswi_addr: usize, mtimer_addr: usize) -> Self {
        Self {
            mswi_address: mswi_addr,
            mtimer_address: mtimer_addr,
        }
    }

    pub fn set_msip(&self, hartid: usize) {
        if hartid > 4094 {
            panic!("hartid cannot be greater than 4094");
        }
        unsafe {
            (self.mswi_address as *mut u32)
                .add(hartid)
                .write_volatile(1)
        }
    }

    pub fn clear_msip(&self, hartid: usize) {
        if hartid > 4094 {
            panic!("hartid cannot be greater than 4094");
        }
        unsafe {
            (self.mswi_address as *mut u32)
                .add(hartid)
                .write_volatile(0)
        }
    }
}
