pub mod frame;
pub mod heap;
pub mod page;
pub mod unit;
pub mod range;

pub fn init() {
    heap::init();
    frame::init();
}
