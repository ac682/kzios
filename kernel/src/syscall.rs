use crate::primitive::uart::Uart;
use crate::qemu::UART;

pub fn forward(id: usize, arg0: usize, arg1: usize, arg2: usize, arg3: usize) {
    match id {
        0 => put_char(arg0),
        _ => todo!("{} not implemented", id),
    };
}

// 0x0
fn put_char(char: usize)
{
    UART.write(char as u8);
}

// 0x1
fn get_char() -> Option<usize>{
    if let Some(char) = UART.read(){
        Some(char as usize)
    }else{
        None
    }
}

// 0x10
fn open(){}

// 0x11
fn read(){}

// 0x12
fn write(){}

// 0x13
fn close(){}

// 0x14
fn delete(){}

// 0x15
fn get_modifier(){}

// 0x16
fn set_modifier(){}

// 0x20
fn fork(){}

// 0x21
fn send_signal(){}

// 0x22
fn exit(){}

// 0x30
fn set_pin(){}

// 0x31
fn digital_write(){}

// 0x32
fn digital_read(){}

// 0x33
fn analog_write(){}

// 0x33
fn analog_read(){}

