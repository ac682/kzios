pub mod hart;

pub trait Timer{
    fn uptime(&self) -> usize;
    fn tick_freq(&self) -> usize;
    fn schedule_next(&mut self, ticks: usize);
}