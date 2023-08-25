pub mod lcg;

pub trait RandomGenerator{
    fn next(&mut self) -> usize;
}