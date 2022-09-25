use crate::primitive::uart::Uart;
use crate::println;
use crate::process::scheduler::exit_process;
use crate::qemu::UART;

pub fn forward(id: u64, arg0: u64, arg1: u64, arg2: u64, arg3: u64) {
    match id {
        0 => do_put_char(arg0 as usize),
        0x22 => do_exit(i64::try_from(arg0).unwrap()),
        _ => todo!("{} not implemented", id),
    };
}

// # system internal
// 0x0
fn do_put_char(char: usize) {
    UART.write(char as u8);
}

// 0x1
fn do_get_char() -> Option<usize> {
    if let Some(char) = UART.read() {
        Some(char as usize)
    } else {
        None
    }
}

// # process
// 0x20
fn do_exit(code: i64) {
    println!("process exit with code {}", code);
    exit_process(code);
}

// 0x2A
fn do_fork() {}

//0x2B
fn do_execute_file() {}

// # signal
// 0x30
fn do_signal_return() {}

// 0x31
fn do_signal_set() {}

// 0x32
fn do_signal_send() {}

// # ipc
// 0x40
fn do_send() {}

// 0x41
fn do_receive() {}
