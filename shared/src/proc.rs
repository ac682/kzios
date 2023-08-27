use alloc::string::String;
use flagset::{flags, FlagSet};
use num_derive::{FromPrimitive, ToPrimitive};

/// ExitCode(i64) type for process
pub type ExitCode = isize;
/// Pid(u32) type for process
pub type Pid = u32;
/// Tid(u32) type for thread
/// If uniform thread-id required, It is uni_tid = ((pid << 32) + tid)
pub type Tid = u32;
/// SignalMap(u64) for process
pub type SignalMap = u64;

flags! {
    /// Predefined signal numbers
    #[derive(FromPrimitive, ToPrimitive)]
    pub enum SystemSignal: SignalMap {
        /// Reserved
        None = 0,
        /// Request to finalize the job and quit. It's a REQUEST! Use kill syscall to finalize a process without notifying
        Terminate = 1 << 0,
        /// Notify the process should check itself for (device interrupts, events listened)
        Notify = 1 << 1,
    }
}

flags! {
    /// Permission of the process
    /// Invalid when fork means copy the permissions from the parent
    pub enum ProcessPermission: u32{
        /// Not available
        Invalid = 0b0,
        /// Should be always present
        Valid = 0b1,
        /// Process operations
        Process = 0b10,
        /// It's a service and can be registered as service
        Service = 0b100,
        /// Map
        Memory = 0b1000,
        /// IDK
        Net = 0b10000,

        /// All of them
        All = (ProcessPermission::Valid | ProcessPermission::Process | ProcessPermission::Service | ProcessPermission::Memory | ProcessPermission::Net).bits()
    }
}

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
/// States of process execution unit
pub enum ExecutionState {
    /// Can be picked as running
    Ready,
    /// Code is being executed
    Running,
    /// Waiting for some signal and need to be waked up
    Waiting(WaitingReason),
    /// Finished, thread would be cleaned up
    Dead,
}

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
/// Waiting reasons
pub enum WaitingReason {
    /// Waken up when time up
    Timer,
    /// Sending message blocks itself
    SendBusy,
    /// Receiving message blocks itself
    ReceiveBusy,
}

/// Process's main function product
pub trait Termination {
    /// Get completed process's exit code
    fn to_exit_code(self) -> ExitCode;
}

impl Termination for () {
    fn to_exit_code(self) -> ExitCode {
        0
    }
}

impl Termination for bool {
    fn to_exit_code(self) -> ExitCode {
        if self {
            0
        } else {
            -1
        }
    }
}

/// ExitCode for process result which treated as Termination
pub type ProgramResult = Result<(), ExitCode>;

impl Termination for ProgramResult {
    fn to_exit_code(self) -> ExitCode {
        if let Err(code) = self {
            code
        } else {
            0
        }
    }
}

/// Process struct for inspect
pub struct ProcessInfo {
    /// Name registered or command line
    pub name: String,
    /// Pid is pid
    pub pid: Pid,
    /// Pid of the parent process of the process
    pub parent: Pid,
    /// State of the process
    pub state: ExecutionState,
    /// Permission of the process
    pub permissions: FlagSet<ProcessPermission>,
}
