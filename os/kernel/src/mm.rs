use erhino_shared::proc::Tid;
use spin::Once;

use crate::{
    external::{_memory_end, _memory_start, _user_trap},
    mm::page::{PageEntryFlag, PageTableEntry, PAGE_BITS},
};

use self::{page::PageEntryImpl, unit::MemoryUnit};

pub mod frame;
pub mod page;
pub mod unit;
pub mod usage;

type KernelUnit = MemoryUnit<PageEntryImpl>;

pub static mut KERNEL_UNIT: Once<KernelUnit> = Once::new();
#[export_name = "_kernel_satp"]
pub static mut KERNEL_SATP: usize = 0;

#[allow(unused)]
pub enum ProcessAddressRegion {
    Invalid,
    Unknown,
    Program,
    Heap,
    Stack(Tid),
    TrapFrame(Tid),
}

pub fn init() {
    let memory_start = _memory_start as usize >> PAGE_BITS;
    let memory_end = _memory_end as usize >> PAGE_BITS;
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
        PageEntryImpl::top_address() >> PAGE_BITS,
        _user_trap as usize >> PAGE_BITS,
        1,
        PageEntryFlag::PrefabKernelTrampoline,
    )
    .expect("map kernel trampoline failed");
    // kernel has no trap frame so it has no trap frame mapped
    let satp = unit.satp();
    unsafe {
        KERNEL_UNIT.call_once(|| unit);
        KERNEL_SATP = satp;
    }
}
