pub mod hart;

pub trait Timer {
    fn get_uptime(&self) -> usize;
    fn get_cycles(&self) -> usize;
    fn set_timer(&mut self, cycles: usize);
    fn ms_to_cycles(&self, time: usize) -> usize;
    fn cycles_to_ms(&self, cycles: usize) -> usize;
}
