pub mod hart;
pub mod mutex;
pub mod optimistic;
pub mod cell;

pub trait Lock<'a, Data: Sized, Guard> {
    fn is_locked(&self) -> bool;
    fn lock(&'a mut self) -> Guard;
    unsafe fn access(&self) -> *const Data;
    unsafe fn access_mut(&mut self) -> *mut Data;
}

pub trait ReadWriteLock<'a, Data: Sized, Guard, MutGuard>: Lock<'a, Data, Guard> {
    fn lock_mut(&'a mut self) -> MutGuard;
}
