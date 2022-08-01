use spin::Mutex;

pub mod heaped;
pub mod paged;

pub fn init() {
    heaped::init();
    paged::init();
}
