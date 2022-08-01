use crate::primitive::uart::Uart;

pub struct NS16550a {
    base_address: usize,
}

impl NS16550a {
    pub fn new() -> Self {
        Self {
            base_address: 0x1000_0000usize,
        }
    }

    pub fn init(&self) {
        unsafe {
            let handler = self.base_address as *mut u8;
            handler.add(3).write_volatile(0b11); // 8 比特的数据位, 0b00 则是 5 位
            handler.add(2).write_volatile(0b1); // enable FIFO
            handler.add(1).write_volatile(0b0); // disable all interrupts
        }
    }
}

impl Uart for NS16550a {
    fn write(&self, char: u8) {
        unsafe {
            let handler = self.base_address as *mut u8;
            handler.add(0).write_volatile(char);
        }
    }

    fn read(&self) -> Option<u8> {
        unsafe {
            let handler = self.base_address as *mut u8;
            // LSR 的第一个比特为 1 则数据到达
            if handler.add(5).read_volatile() & 0b1 == 0 {
                None
            } else {
                Some(handler.add(0).read_volatile())
            }
        }
    }
}
