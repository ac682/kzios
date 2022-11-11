use core::ops::{Deref, DerefMut};

use spin::Once;

pub mod hart;
pub mod mutex;
pub mod optimistic;

pub trait InteriorLock {
    fn is_locked(&self) -> bool;
    fn lock(&mut self);
    fn try_lock(&mut self) -> bool;
    fn unlock(&mut self);
}

pub trait InteriorReadWriteLock: InteriorLock {
    fn lock_mut(&mut self);
    fn try_lock_mut(&mut self) -> bool;
}

pub struct DataLock<Data: Sized, Lock: InteriorLock> {
    inner: Lock,
    data: Data,
}

pub struct DataLockGuard<'lock, Data: Sized, Lock: InteriorLock> {
    locked: &'lock mut Lock,
    data: *mut Data,
}

unsafe impl<Data: Sized, Lock: InteriorLock> Sync for DataLock<Data, Lock> {}

impl<Data: Sized, Lock: InteriorLock> DataLock<Data, Lock> {
    pub const fn new(data: Data, lock: Lock) -> Self {
        Self { inner: lock, data }
    }

    pub fn lock(&'static mut self) -> DataLockGuard<Data, Lock> {
        self.inner.lock();
        DataLockGuard {
            locked: &mut self.inner,
            data: &mut self.data as *mut Data,
        }
    }
}

impl<'lock, Data: Sized, Lock: InteriorLock> Deref for DataLockGuard<'lock, Data, Lock> {
    type Target = Data;

    fn deref(&self) -> &Self::Target {
        unsafe { &*self.data }
    }
}

impl<'lock, Data: Sized, Lock: InteriorLock> DerefMut for DataLockGuard<'lock, Data, Lock> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        unsafe { &mut *self.data }
    }
}

impl<'lock, Data: Sized, Lock: InteriorLock> Drop for DataLockGuard<'lock, Data, Lock> {
    fn drop(&mut self) {
        self.locked.unlock();
    }
}

pub struct LazyDataLock<Data: Sized, Lock: InteriorLock> {
    data: Once<Data>,
    lock: Lock,
}

unsafe impl<Data: Sized, Lock: InteriorLock> Sync for LazyDataLock<Data, Lock> {}

impl<Data: Sized, Lock: InteriorLock> LazyDataLock<Data, Lock> {
    pub const fn new(lock: Lock) -> Self {
        Self {
            data: Once::new(),
            lock,
        }
    }

    pub fn put(&mut self, data: Data) {
        self.data.call_once(|| data);
    }

    pub fn lock(&'static mut self) -> DataLockGuard<Data, Lock> {
        self.lock.lock();
        DataLockGuard {
            locked: &mut self.lock,
            data: self.data.as_mut_ptr(),
        }
    }
}
