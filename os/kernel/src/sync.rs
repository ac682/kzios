use core::{ops::{Deref, DerefMut}};

pub mod hart;
pub mod mutex;

pub trait InteriorLock {
    fn is_locked(&self) -> bool;
    fn lock(& mut self);
    fn try_lock(&mut self) -> bool;
    fn unlock(&mut self);
}

pub trait InteriorReadWriteLock: InteriorLock{
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

impl<Data: Sized, Lock: InteriorLock> DataLock<Data, Lock> {
    pub const fn new(data: Data, lock: Lock) -> Self {
        Self {
            inner: lock,
            data,
        }
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

impl<'lock, Data:Sized, Lock:InteriorLock> Drop for DataLockGuard<'lock, Data,Lock>{
    fn drop(&mut self) {
        self.locked.unlock();
    }
}