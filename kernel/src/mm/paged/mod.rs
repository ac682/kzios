use core::arch::asm;

use riscv::register::satp::{self, Mode};

use self::{
    address::{PhysicalAddress, VirtualAddress},
    frame_allocator::frame_alloc,
    page_table::{map, PageTable},
};

pub mod address;
pub mod frame_allocator;
mod page_table;

pub fn init() {
    let frame = frame_alloc().unwrap();
    let kernel_table = PageTable::new(2, frame);
    // 只有内核态的页表需要等值映射用于驱动
    //TODO: 地址应该从设备树获得
    // 直接全部映射！ U X R V 都是1, X 其实应该单独设置
    // 就映射内核地址，不然不够用啦
    map(
        &kernel_table,
        VirtualAddress::from(0x80200000u64),
        PhysicalAddress::from(0x80200000u64),
        0x20000,
        0b1111,
    );
    // 开启 satp
    unsafe {
        satp::set(Mode::Sv39, 0, u64::from(frame) as usize);
    }
    test();
}

fn test(){
    println!("SATP On!");
}
