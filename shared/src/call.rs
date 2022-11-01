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
    // Signal
    /// Return from signal handler
    SignalReturn = 0x20,
    /// Send a signal to the process
    SignalSend = 0x21,
    /// Set signal handler for the current process
    SignalSet = 0x22,
    // IPC
    /// Send a message carrying a huge payload then block until message received
    Send = 0x30,
    /// Block and check if a message enter then retrieve payload
    Receive = 0x31,
    /// IDK
    Notify = 0x32,
    // Process memory
    /// Map a range of virtual addresses for the process with kernel served pages
    Extend = 0x40,
    /// Map a range of virtual addresses for the process with specific range of physical addresses
    Map = 0x41,
    /// Discard and tell kernel to reuse a range of virtual addresses mapped before
    Free = 0x42,
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
