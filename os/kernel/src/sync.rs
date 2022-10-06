pub mod hart;
pub mod mutex;
pub mod optimistic;
pub mod spin;

pub trait Lock<Data: Sized, Guard> {
    fn lock(&self) -> Guard;
}
