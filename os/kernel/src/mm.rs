pub mod frame;
pub mod heap;
pub mod page;
pub mod unit;

pub fn init() {
    heap::init();
}
