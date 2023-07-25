pub mod smooth;

pub trait Scheduler{
    fn add(&mut self);
    fn schedule(&mut self);
}