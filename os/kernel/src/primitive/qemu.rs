use alloc::sync::Arc;

use crate::primitive::qemu::ns16550a::NS16550a;

pub mod ns16550a;

lazy_static! {
    pub static ref UART: Arc<NS16550a> = Arc::new(NS16550a::new());
}

pub fn init() {
    UART.init();
}
