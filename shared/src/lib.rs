#![no_std]
#![warn(missing_docs)]

//! # eRhino shared lib
//! 
//! Predefined types and system calls

/// ExitCode(i32) type for process
pub type ExitCode = i32;
/// Pid(u32) type for process
pub type Pid = u32;
/// Address(u64) type for process
pub type Address = u64;
/// PageNumber(u64) for process
pub type PageNumber = u64;
/// SignalNumber(u64) for process
pub type SignalNo = u64;

/// Predefined signal numbers
#[repr(u64)]
pub enum Signal {
    /// Do nothing
    Nop = 0x1,
    /// Interrupt current workflow but not quit
    Interrupt = 0x2,
    /// Finalize the job and quit
    Terminate = 0x3,
}

/// Predefined system calls
/// 
/// Not available for userspace
/// Using service call instead
#[repr(u64)]
pub enum SystemCall{
    // System reserved
    /// Write to board defined output stream
    Write = 0x0,
    /// Read from board defined input stream
    Read = 0x1,
    // Process control
    /// Finalized process notifies kernel to cleanup
    Exit = 0x10,
    /// Send a signal to the other processes
    Signal = 0x11,
    /// Fork process itself
    Fork = 0x12,
    /// Wait for all child processes to exit
    Wait = 0x13,
    /// Wait for a certain process to exit
    WaitFor = 0x14,
    /// Replace the process's execution image with the new one from the bytes
    ExecuteBytes = 0x1A,
    /// Replace the process's execution image with the new one from the file
    ExecuteFile = 0x1B,
    // IPC
    /// Send a message carrying a huge payload
    Send = 0x20,
    /// Prepared to receive a message and enter receiving procedure
    Receive = 0x21,
    /// IDK
    Notify = 0x22,
    // Process memory
    /// Map a range of virtual addresses for the process with kernel served pages
    Map = 0x30,
    /// discard and tell kernel to reuse a range of virtual addresses mapped before 
    Free = 0x31,
}

/// Predefined kernel calls (aka trap calls)
/// 
/// Not available for userspace
/// Using service call instead
#[repr(u64)]
pub enum KernelCall{
    /// Enter user mode and begin scheduling
    EnterUserSpace = 0x0
}