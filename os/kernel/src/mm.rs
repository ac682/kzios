use riscv::register::satp;
use spin::Once;

use crate::{external::{_memory_start, _memory_end, _trampoline}, mm::page::{PageTableEntry39, PageEntryFlag, PageTableEntry}, println};

use self::{unit::MemoryUnit, page::PageEntryImpl};

pub mod frame;
pub mod page;
pub mod unit;
pub mod layout;

type KernelUnit = MemoryUnit<PageEntryImpl>;

static mut KERNEL_UNIT: Once<KernelUnit> = Once::new();
#[export_name = "_kernel_satp"]
pub static mut KERNEL_SATP: usize = 0;


pub fn init() {
    let memory_start = _memory_start as usize >> 12;
    let memory_end = _memory_end as usize >> 12;
    let mut unit = MemoryUnit::<PageEntryImpl>::new().unwrap();
    // mmio device space
    unit.map(
        0x0,
        0x0,
        memory_start,
        PageEntryFlag::Valid | PageEntryFlag::Writeable | PageEntryFlag::Readable,
    )
    .expect("map mmio device failed");
    // sbi + kernel space
    unit.map(
        memory_start,
        memory_start,
        memory_end - memory_start,
        PageEntryFlag::Valid
            | PageEntryFlag::Writeable
            | PageEntryFlag::Readable
            | PageEntryFlag::Executable,
    )
    .expect("map sbi + kernel space failed");
    let top_address = PageEntryImpl::top_address();
    // trampoline code page
    unit.map(
        top_address >> 12,
        _trampoline as usize >> 12,
        1,
        PageEntryFlag::Valid
            | PageEntryFlag::Writeable
            | PageEntryFlag::Readable
            | PageEntryFlag::Executable,
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
