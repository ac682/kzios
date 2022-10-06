pub mod hart;
pub mod mutex;
pub mod optimistic;
pub mod spin;

pub trait Lock<'a, Data: Sized, Guard> {
    fn lock(&'a mut self) -> Guard;
}
