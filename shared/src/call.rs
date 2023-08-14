use num_derive::{FromPrimitive, ToPrimitive};

/// Predefined system call errors
#[repr(usize)]
#[derive(Debug, FromPrimitive, ToPrimitive)]
pub enum SystemCallError {
    // Generic errors
    /// [SystemCallError::NoError] means no errors at all
    NoError = 0x00,
    /// Undefined error
    Unknown = 0x01,
    /// Undefined error
    InternalError = 0x02,
    /// Argument out of range or illegal
    IllegalArgument = 0x3,
    // Role of process
    /// Process must need the permission to do the system call
    PermissionRequired = 0x10,
    // Memory related
    /// System is out of memory or the process reached the allocation limit
    OutOfMemory = 0x20,
    /// Address is not power of two or page-aligned
    MisalignedAddress = 0x21,
    /// The region accessed is not available
    MemoryNotAccessible = 0x22,
}

/// Predefined system calls
///
/// Only accessible in userspace
/// ipc_call is sent through SystemCall::IPC
#[repr(usize)]
#[derive(Debug, FromPrimitive, ToPrimitive)]
pub enum SystemCall {
    // System reserved
    /// Undefined behavior in release environment
    Debug = 0x00,
    /// Write to defined output stream
    Write = 0x01,
    /// Read from defined input stream
    Read = 0x02,
    // Process control
    /// Finalized process notifies kernel to cleanup
    Exit = 0x10,
    /// Fetch a process's information and fill in the [super::proc::ProcessInfo] struct
    Inspect = 0x14,
    /// Fetch the current process's information and fill in the [super::proc::ProcessInfo] struct
    InspectMyself = 0x16,
    /// Replace the process's execution image with the new one from the bytes
    ExecuteBytes = 0x1A,
    /// Replace the process's execution image with the new one from the file
    ExecuteFile = 0x1B,
    // Thread
    /// Finalized thread notifies kernel to cleanup
    ThreadExit = 0x20,
    /// Be nice
    ThreadYield = 0x21,
    /// Create a thread for the process
    ThreadSpawn = 0x22,
    /// Wait another owned thread to exit
    ThreadJoin = 0x23,
    /// Kill owned thread
    ThreadKill = 0x24,
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
    /// Aka. IOMap, Memory permission required
    Map = 0x51,
    /// Discard and tell kernel to reuse a range of virtual addresses
    Free = 0x52,
}
