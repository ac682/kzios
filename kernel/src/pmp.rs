use riscv::register::{Permission, Pmp, Pmpcsr, Range};
use riscv::register::{pmpaddr0, pmpaddr1, pmpaddr2, pmpaddr3, pmpcfg0};
use riscv::register::scause::{set, Trap};

use crate::external::{_kernel_end, _kernel_start, _memory_end, _memory_start};

pub fn init() {
    unsafe {
        pmpcfg0::set_pmp(0, Range::OFF, Permission::NONE, false);
        pmpaddr0::write(0);
        // 外设
        pmpcfg0::set_pmp(1, Range::TOR, Permission::RW, false);
        pmpaddr1::write(_memory_start as usize >> 2);
        // RAM
        pmpcfg0::set_pmp(2, Range::TOR, Permission::RWX, false);
        pmpaddr2::write(_memory_end as usize >> 2);
    }
}