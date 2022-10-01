use core::usize;

use flagset::FlagSet;

use crate::mm::paged::page_table::PageTableEntryFlag;
use crate::primitive::uart::Uart;
use crate::println;
use crate::process::scheduler::{self, add_process, exit_process, trap_with_current};
use crate::qemu::UART;
use crate::typedef::*;

pub fn forward(id: u64, arg0: u64, arg1: u64, arg2: u64, arg3: u64) {
    match id {
        0x00 => do_write(arg0 as usize),
        0x20 => do_exit(arg0 as ExitCode),
        0x21 => do_fork(),
        0x31 => do_signal_set(arg0 as Address),
        0x50 => do_map(arg0, arg1 as usize, arg2),
        _ => todo!("{} not implemented", id),
    }
}

// 0x00
fn do_write(char: usize) {
    UART.write(char as u8);
}

// 0x01
fn do_read() -> Option<usize> {
    if let Some(char) = UART.read() {
        Some(char as usize)
    } else {
        None
    }
}

fn do_write_line(addr: Address, line: usize) {
    todo!()
}

// # process
// 0x20
fn do_exit(code: ExitCode) {
    exit_process(code);
}

// 0x21
// + Pid<u32> for child process
// 0 Parent self
// - Errno:
// -1 -> udf
// -2 -> udf
fn do_fork() {
    trap_with_current(|parent| {
        let mut child = parent.fork();
        child.move_to_next_instruction();
        child.set_return_value_in_register(0u64);
        let new_pid = add_process(child);
        parent.set_return_value_in_register(new_pid as u64);
    })
}

// 0x22
fn do_wait_child(pid: Pid) {
    todo!()
}

fn do_wait_children() {
    todo!()
}

//0x2B
fn do_execute_file() {}

// # signal
// 0x30
fn do_signal_return() {}

// 0x31
fn do_signal_set(handler_address: Address) {
    trap_with_current(|proc| {
        proc.set_signal_handler(handler_address);
    });
}

// 0x32
fn do_signal_send(pid: Pid, signal: SignalNo) {
    
}

// # ipc
// 0x40
fn do_send() {}

// 0x41
fn do_receive() {}

// # mm
// 0x50
fn do_map(vpn: PageNumber, count: usize, flags: u64) {
    trap_with_current(|proc| {
        proc.map(
            vpn,
            count,
            FlagSet::<PageTableEntryFlag>::new(flags).unwrap()
                | PageTableEntryFlag::User
                | PageTableEntryFlag::Valid,
        );
    })
}
