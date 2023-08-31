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
    /// System call can not be performed
    FunctionNotAvailable = 0x04,
    // Role of process
    /// Process must need the permission to do the system call
    PermissionDenied = 0x10,
    // Memory related
    /// System is out of memory or the process reached the allocation limit
    OutOfMemory = 0x20,
    /// Address is not power of two or page-aligned
    MisalignedAddress = 0x21,
    /// The region accessed is not available
    MemoryNotAccessible = 0x22,
    // Special operations
    /// Specific operation cannot be applied due to bad reference
    ObjectNotFound = 0x30,
    /// Found but owned by others
    ObjectNotAccessible = 0x31,
    /// Can not own more objects
    ReachLimit = 0x32
}

/// Predefined system calls
///
/// Only accessible in userspace
/// ipc_call is sent through SystemCall::IPC
#[repr(usize)]
#[derive(Debug, FromPrimitive, ToPrimitive, Clone, Copy)]
pub enum SystemCall {
    // System reserved
    /// Undefined behavior in release environment
    Debug = 0x00,
    // Process control
    /// Finalized process notifies kernel to cleanup
    Exit = 0x10,
    /// Fetch a process's information and fill in the [super::proc::ProcessInfo] struct
    Inspect = 0x14,
    /// Fetch the current process's information and fill in the [super::proc::ProcessInfo] struct
    InspectMyself = 0x15,
    /// Spawn a process from the given bytes
    ExecuteBytes = 0x16,
    /// Spawn a process from the file
    ExecuteFile = 0x17,
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
    // Messaging
    /// Send a message carrying a huge payload then block until message received
    Send = 0x40,
    /// Block and check if a message enter then retrieve payload
    Receive = 0x41,
    /// IDK
    Notify = 0x42,
    // Process memory
    /// Map a range of virtual addresses for the process with kernel served pages
    Extend = 0x50,
    /// Map a range of virtual addresses for the process with specific range of physical addresses
    /// Aka. IOMap, Memory permission required
    Map = 0x51,
    /// Discard and tell kernel to reuse a range of virtual addresses
    Free = 0x52,
    //  Tunnel
    /// Allocate a key-marked random page
    TunnelBuild = 0x60,
    /// Link a allocated page with a key
    TunnelLink = 0x61,
    /// Dispose the tunnel and restore the slot
    TunnelDispose = 0x62,
}
