pub mod hart;
pub mod mutex;
pub mod optimistic;

pub trait Lock<'a, Data: Sized, Guard> {
    fn lock(&'a mut self) -> Guard;
}

pub trait ReadWriteLock<'a, Data: Sized, Guard, MutGuard>: Lock<'a, Data, Guard>{
    fn lock_mut(&'a mut self) -> MutGuard;
}