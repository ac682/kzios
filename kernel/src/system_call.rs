use crate::primitive::uart::Uart;
use crate::println;
use crate::process::scheduler::exit_process;
use crate::qemu::UART;

pub fn forward(id: u64, arg0: u64, arg1: u64, arg2: u64, arg3: u64) {
    match id {
        0 => put_char(arg0 as usize),
        0x22 => exit(i64::try_from(arg0).unwrap()),
        _ => todo!("{} not implemented", id),
    };
}

// # system internal
// 0x0
fn put_char(char: usize) {
    UART.write(char as u8);
}

// 0x1
fn get_char() -> Option<usize> {
    if let Some(char) = UART.read() {
        Some(char as usize)
    } else {
        None
    }
}

// # process
// 0x20
fn fork() {}

// 0x21
fn send_signal() {}

// 0x22
fn exit(code: i64) {
    exit_process(code);
}

// # ipc
// 0x30
fn send() {}

// 0x31
fn receive() {}
