use core::hint::spin_loop;

use erhino_shared::proc::Tid;
use riscv::register::satp;
use spin::Once;

use crate::{
    external::{_memory_end, _memory_start, _user_trap},
    mm::page::{PageEntryFlag, PageTableEntry, PageTableEntry39},
    println,
};

use self::{page::PageEntryImpl, unit::MemoryUnit};

pub mod frame;
pub mod page;
pub mod unit;

type KernelUnit = MemoryUnit<PageEntryImpl>;

static mut KERNEL_UNIT: Once<KernelUnit> = Once::new();
#[export_name = "_kernel_satp"]
pub static mut KERNEL_SATP: usize = 0;

#[derive(Debug)]
pub enum MemoryOperation {
    Read,
    Write,
    Execute,
}

pub enum ProcessAddressRegion {
    Invalid,
    Unknown,
    Program,
    Heap,
    Stack(Tid),
    TrapFrame(Tid),
}

pub fn init() {
    let memory_start = _memory_start as usize >> 12;
    let memory_end = _memory_end as usize >> 12;
    let mut unit = MemoryUnit::<PageEntryImpl>::new().unwrap();
    // mmio device space
    unit.map(0x0, 0x0, memory_start, PageEntryFlag::PrefabKernelDevice)
        .expect("map mmio device failed");
    // sbi + kernel space
    unit.map(
        memory_start,
        memory_start,
        memory_end - memory_start,
        PageEntryFlag::PrefabKernelProgram,
    )
    .expect("map sbi + kernel space failed");
    // trampoline
    unit.map(
        PageEntryImpl::top_address() >> 12,
        _user_trap as usize >> 12,
        1,
        PageEntryFlag::PrefabKernelTrampoline,
    )
    .expect("map kernel trampoline failed");
    // kernel has no trap frame so it has no trap frame mapped
    println!("{}", unit);
    let satp = unit.satp();
    unsafe {
        KERNEL_UNIT.call_once(|| unit);
        KERNEL_SATP = satp;
    }
    satp::write(satp);
}
