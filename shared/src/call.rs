use num_derive::{ToPrimitive, FromPrimitive};

/// Predefined system calls
///
/// Only accessible in userspace
/// ipc_call is done through SystemCall::IPC part
#[repr(usize)]
#[derive(FromPrimitive, ToPrimitive)]
pub enum SystemCall {
    // System reserved
    /// Write to board defined output stream
    Write = 0x0,
    /// Read from board defined input stream
    Read = 0x1,
    // Process control
    /// Finalized process notifies kernel to cleanup
    Exit = 0x10,
    /// Yield return
    Yield = 0x11,
    /// Send a signal to the other processes
    Signal = 0x12,
    /// Fork process itself
    Fork = 0x13,
    /// Wait for all child processes to exit
    Wait = 0x14,
    /// Wait for a certain process to exit
    WaitFor = 0x15,
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
    Extend = 0x30,
    /// Map a range of virtual addresses for the process with specific range of physical addresses
    Map = 0x31,
    /// Discard and tell kernel to reuse a range of virtual addresses mapped before
    Free = 0x32,
}

/// Predefined kernel calls (aka trap calls)
///
/// Not available for userspace
/// Using service call instead
#[repr(usize)]
#[derive(FromPrimitive, ToPrimitive)]
pub enum KernelCall {
    /// Enter user mode and begin scheduling
    EnterUserSpace = 0x0,
}
