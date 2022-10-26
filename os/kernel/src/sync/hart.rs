use core::{
    ops::{Deref, DerefMut},
    sync::atomic::{AtomicU64, Ordering},
};

use riscv::{asm::nop, register::mhartid};
use spin::Once;

use super::{Lock, ReadWriteLock};

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

unsafe impl<T> Sync for HartLock<T> {}

impl<'a, T> Lock<'a, T, HartLockGuard<'a, T>> for HartLock<T> {
    fn is_locked(&self) -> bool{
        let locked = self.locked.load(Ordering::Relaxed);
        locked != u64::MAX && locked != mhartid::read() as u64
    }

    fn lock(&'a mut self) -> HartLockGuard<'a, T> {
        let hartid = mhartid::read() as u64;
        while self
            .locked
            .compare_exchange(u64::MAX, hartid, Ordering::Acquire, Ordering::Relaxed)
            .is_err_and(|current| current != hartid)
        {
            // spin
            unsafe { nop() }
        }
        HartLockGuard {
            data: self.data.as_mut_ptr(),
            locked: &mut self.locked,
        }
    }

    unsafe fn access(&self) -> *const T{
        self.data.get_unchecked()
    }

    unsafe fn access_mut(&mut self) -> *mut T{
        self.data.as_mut_ptr()
    }
}

impl<'a, T> Drop for HartLockGuard<'a, T> {
    fn drop(&mut self) {
        let hartid = mhartid::read() as u64;
        while self
            .locked
            .compare_exchange(hartid, u64::MAX, Ordering::Release, Ordering::Relaxed)
            .is_err_and(|current| current != u64::MAX)
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

pub struct HartReadWriteLock<T: Sized> {
    data: Option<T>,
    locked: AtomicU64,
}

unsafe impl<T> Sync for HartReadWriteLock<T> {}

pub struct HartReadLockGuard<T: Sized> {
    data: *const T,
}

pub struct HartWriteLockGuard<'a, T: Sized + 'a> {
    data: *mut T,
    locked: &'a AtomicU64,
}

impl<T> HartReadWriteLock<T> {
    pub const fn new(data: T) -> Self{
        Self{
            data: Some(data),
            locked: AtomicU64::new(u64::MAX)
        }
    }

    pub const fn empty() -> Self {
        Self {
            data: None,
            locked: AtomicU64::new(u64::MAX),
        }
    }

    pub fn put(&mut self, data: T) {
        self.data = Some(data);
    }
}

impl<'a, T> Drop for HartWriteLockGuard<'a, T> {
    fn drop(&mut self) {
        let hartid = mhartid::read() as u64;
        while self
            .locked
            .compare_exchange(hartid, u64::MAX, Ordering::Release, Ordering::Relaxed)
            .is_err_and(|current| current != u64::MAX)
        {
            // 在这里自旋是没有意义的，其他 hart 必定会卡在获得锁的自旋处而无法到达此处
            unsafe { nop() }
        }
    }
}

impl<T> Deref for HartReadLockGuard<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        unsafe { &*self.data }
    }
}

impl<'a, T> Deref for HartWriteLockGuard<'a, T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        unsafe { &*self.data }
    }
}

impl<'a, T> DerefMut for HartWriteLockGuard<'a, T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        unsafe { &mut *self.data }
    }
}

impl<'a, T> ReadWriteLock<'a, T, HartReadLockGuard<T>, HartWriteLockGuard<'a, T>>
    for HartReadWriteLock<T>
{
    fn lock_mut(&'a mut self) -> HartWriteLockGuard<'a, T> {
        let hartid = mhartid::read() as u64;
        while self
            .locked
            .compare_exchange(u64::MAX, hartid, Ordering::Acquire, Ordering::Relaxed)
            .is_err_and(|current| current != hartid)
        {
            unsafe { nop() }
        }
        HartWriteLockGuard {
            data: self.data.as_mut().unwrap(),
            locked: &mut self.locked,
        }
    }
}

impl<'a, T> Lock<'a, T, HartReadLockGuard<T>> for HartReadWriteLock<T> {
    fn is_locked(&self) -> bool{
        let locked = self.locked.load(Ordering::Relaxed);
        locked != u64::MAX && locked != mhartid::read() as u64
    }

    fn lock(&'a mut self) -> HartReadLockGuard<T> {
        let hartid = mhartid::read() as u64;
        while {
            let current = self.locked.load(Ordering::Acquire);
            !(current == hartid || current == u64::MAX)
        } {
            unsafe { nop() }
        }
        HartReadLockGuard {
            data: self.data.as_mut().unwrap(),
        }
    }

    unsafe fn access(&self) -> *const T {
        self.data.as_ref().unwrap()
    }

    unsafe fn access_mut(&mut self) -> *mut T{
        self.data.as_mut().unwrap()
    }
}
