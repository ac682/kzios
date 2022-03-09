#![allow(unused)]

pub const USER_STACK_SIZE: usize = 4096 * 2;
pub const KERNEL_STACK_SIZE: usize = 4096 * 2;
pub const KERNEL_HEAP_SIZE: usize = 0x1_0000; // 40KB, 原先0x20_0000, 2MB, 实际没必要这么大
pub const MEMORY_END: usize = 0x8060_0000; // 8MB when kpu disabled
pub const PAGE_SIZE: usize = 0x1000; // 4KB, 页号需要乘以页尺寸来确定实际地址大的偏移量
pub const PAGE_SIZE_BITS: usize = 12; // 12位，用于 shift
pub const PAGE_TABLE_SIZE: usize = 512;

pub const TRAMPOLINE: usize = usize::MAX - PAGE_SIZE + 1;
pub const TRAP_CONTEXT: usize = TRAMPOLINE - PAGE_SIZE;
