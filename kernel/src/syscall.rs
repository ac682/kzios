use crate::primitive::uart::Uart;
use crate::qemu::UART;

pub fn forward(id: usize, arg0: usize, arg1: usize, arg2: usize, arg3: usize) {
    match id {
        0 => put(arg0),
        _ => todo!("{} not implemented", id),
    };
}

fn put(char: usize)
{
    UART.write(char as u8);
}