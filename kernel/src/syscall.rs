use crate::primitive::uart::Uart;
use crate::qemu::UART;

pub fn forward(id: usize, arg0: usize, arg1: usize, arg2: usize, arg3: usize) {
    match id {
        0 => put_char(arg0),
        _ => todo!("{} not implemented", id),
    };
}

// 0
fn put_char(char: usize)
{
    UART.write(char as u8);
}

// 1
fn get_char() -> Option<usize>{
    if let Some(char) = UART.read(){
        Some(char as usize)
    }else{
        None
    }
}

// 10
fn open(){}

// 11
fn read(){}

// 12
fn write(){}

// 13
fn close(){}

// 14
fn delete(){}

// 15
fn get_modifier(){}

// 16
fn set_modifier(){}

// 20
fn fork(){}

// 21
fn send_signal(){}

// 22
fn exit(){}

