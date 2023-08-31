pub mod cpu;

pub trait Timer {
    fn uptime(&self) -> usize;
    fn schedule_next(&mut self, ms: usize);
    fn put_off(&mut self);
}
