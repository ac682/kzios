use core::{
    ops::{Deref, DerefMut},
    sync::atomic::{AtomicBool, AtomicU64, Ordering},
};

use riscv::{
    asm::{delay, nop},
    register::mhartid,
};
use spin::Once;

use super::Lock;

/// 和 [spin::mutex::Spin] 差不多是同一个东西，功能和效果却完全相同
/// 前者对于能对于任何线程实现锁的效果，而这个在同一个 hart 时能跳过自旋直接获得锁。由于本内核没有内核线程*，所以不存在一个 hart 在没释放锁时又去获得锁的情况，故两者等价。
/// 
/// *内核代码执行全部位于陷入上下文中且不可打断，内核的多个引导过程都由引导核完成，也不存在并发
pub struct HartLock<T: Sized> {
    data: Once<T>,
    locked: AtomicU64,
}

pub struct HartLockGuard<'a, T: Sized + 'a> {
    data: *mut T,
    locked: &'a mut AtomicU64,
}

impl<T> HartLock<T> {
    pub const fn empty() -> Self {
        Self {
            data: Once::new(),
            locked: AtomicU64::new(u64::MAX),
        }
    }

    pub fn put(&self, data: T) {
        self.data.call_once(|| data);
    }
}

impl<'a, T> Lock<'a, T, HartLockGuard<'a, T>> for HartLock<T> {
    fn lock(&'a mut self) -> HartLockGuard<'a, T> {
        let hartid = mhartid::read() as u64;
        while self
            .locked
            .compare_exchange(u64::MAX, hartid, Ordering::Acquire, Ordering::Relaxed)
            .is_err()
        {
            // spin
            unsafe { nop() }
        }
        HartLockGuard {
            data: self.data.as_mut_ptr(),
            locked: &mut self.locked,
        }
    }
}

impl<'a, T> Drop for HartLockGuard<'a, T> {
    fn drop(&mut self) {
        let hartid = mhartid::read() as u64;
        while self
            .locked
            .compare_exchange(hartid, u64::MAX, Ordering::Release, Ordering::Relaxed)
            .is_err()
        {
            // 在这里自旋是没有意义的，其他 hart 必定会卡在获得锁的自旋处而无法到达此处
            unsafe { nop() }
        }
    }
}

impl<'a, T> Deref for HartLockGuard<'a, T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        unsafe { &*self.data }
    }
}

impl<'a, T> DerefMut for HartLockGuard<'a, T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        unsafe { &mut *self.data }
    }
}
