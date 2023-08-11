use erhino_shared::mem::Address;
use riscv::register::scause::Scause;

use crate::trap::TrapFrame;

extern "C" {
    pub fn _hart_num();
    pub fn _memory_start();
    pub fn _kernel_start();
    pub fn _bss_start();
    pub fn _bss_end();
    pub fn _heap_start();
    pub fn _stack_start();
    pub fn _kernel_end();
    pub fn _memory_end();
    
    pub fn _trampoline(trapframe: &TrapFrame, cause: usize, val: usize);
    pub fn _park() -> !;
    pub fn _switch(user_trap: Address, satp: usize, trapframe: usize) -> !;
    pub fn _stack_size();
}
