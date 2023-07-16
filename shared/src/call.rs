use num_derive::{FromPrimitive, ToPrimitive};

/// Predefined system calls
///
/// Only accessible in userspace
/// ipc_call is done through SystemCall::IPC part
#[repr(usize)]
#[derive(FromPrimitive, ToPrimitive)]
pub enum SystemCall {
    // System reserved
    /// Undefined behavior in release environment
    Debug = 0x00,
    /// Write to board defined output stream
    Write = 0x01,
    /// Read from board defined input stream
    Read = 0x02,
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
    /// Fetch a process's information and fill in the [super::proc::ProcessInfo] struct
    Inspect = 0x15,
    /// Fetch the current process's information and fill in the [super::proc::ProcessInfo] struct
    InspectMyself = 0x16,
    /// Replace the process's execution image with the new one from the bytes
    ExecuteBytes = 0x1A,
    /// Replace the process's execution image with the new one from the file
    ExecuteFile = 0x1B,
    // Thread
    /// Create a thread for the process
    ThreadSpawn = 0x20,
    /// Wait another owned thread to exit
    ThreadJoin = 0x21,
    /// Kill owned thread
    ThreadKill = 0x22,
    // Signal
    /// Return from signal handler
    SignalReturn = 0x30,
    /// Send a signal to the process
    SignalSend = 0x31,
    /// Set signal handler for the current process
    SignalSet = 0x32,
    // IPC
    /// Send a message carrying a huge payload then block until message received
    Send = 0x40,
    /// Block and check if a message enter then retrieve payload
    Receive = 0x41,
    /// IDK
    Notify = 0x42,
    // IPC for services
    /// Register a service. Requires Service permission
    ServiceRegister = 0x4A,
    /// Find a service's pid
    ServiceQuery = 0x4B,
    // Process memory
    /// Map a range of virtual addresses for the process with kernel served pages
    Extend = 0x60,
    /// Map a range of virtual addresses for the process with specific range of physical addresses
    Map = 0x51,
    /// Discard and tell kernel to reuse a range of virtual addresses mapped before
    Free = 0x52,
}
