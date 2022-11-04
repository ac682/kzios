use core::{ops::{Deref, DerefMut}, marker::PhantomData};

pub mod hart;
pub mod mutex;

pub trait InteriorLock<'lock> {
    type Guard;
    fn is_locked(&self) -> bool;
    fn lock(&'lock mut self) -> Self::Guard;
}

pub trait InteriorReadWriteLock<'lock>: InteriorLock<'lock>{
    type GuardMut;

    fn lock_mut(&'lock mut self) -> Self::GuardMut;
}

pub struct DataLock<Data: Sized, Lock: InteriorLock<'static>> {
    inner: Lock,
    data: Data,
}

pub struct DataLockGuard<'lock, Data: Sized, Lock: InteriorLock<'lock>> {
    inner: Lock::Guard,
    data: *mut Data,
}

impl<Data: Sized, Lock: InteriorLock<'static>> DataLock<Data, Lock> {
    pub const fn new(data: Data, lock: Lock) -> Self {
        Self {
            inner: lock,
            data,
        }
    }

    pub fn lock(&'static mut self) -> DataLockGuard<Data, Lock> {
        DataLockGuard {
            inner: self.inner.lock(),
            data: &mut self.data as *mut Data,
        }
    }
}

impl<'lock, Data: Sized, Lock: InteriorLock<'lock>> Deref for DataLockGuard<'lock, Data, Lock> {
    type Target = Data;

    fn deref(&self) -> &Self::Target {
        unsafe { &*self.data }
    }
}

impl<'lock, Data: Sized, Lock: InteriorLock<'lock>> DerefMut for DataLockGuard<'lock, Data, Lock> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        unsafe { &mut *self.data }
    }
}


