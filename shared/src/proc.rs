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
pub type Signal = u64;

#[repr(u64)]
#[derive(FromPrimitive, ToPrimitive)]
/// Predefined signal numbers
pub enum SystemSignal {
    /// Do nothing
    Nop = 0b1,
    /// Interrupt current workflow but not quit
    Interrupt = 0b10,
    /// Request to finalize the job and quit
    Terminate = 0b100,
    /// Kill the process after signal handled within a period of time
    Stop = 0b1000,
}

flags! {
    /// Permission of the process
    /// Invalid when fork means copy the permissions from the parend
    pub enum ProcessPermission: u8{
        /// Not available
        Invalid = 0b0,
        /// Should be always present
        Valid = 0b1,
        /// Process operations
        Process = 0b10,
        /// Map
        Memory = 0b100,
        /// IDK
        Net = 0b1000,

        /// All of them
        All = (ProcessPermission::Valid | ProcessPermission::Process | ProcessPermission::Memory | ProcessPermission::Net).bits()
    }
}

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
/// States of process
pub enum ProcessState {
    /// Can be picked as running process
    Ready,
    /// Code is being executed
    Running,
    /// Waiting for some signal and need to be waked up
    Waiting(WaitingReason),
    /// Finished, process would be cleaned up and pid put into recycling
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

/// Process struct for inspect
pub struct ProcessInfo {
    /// Name registered or command line
    pub name: String,
    /// Pid is pid
    pub pid: Pid,
    /// Pid of the parent process of the process
    pub parent: Pid,
    /// State of the process
    pub state: ProcessState,
    /// Permission of the process
    pub permissions: FlagSet<ProcessPermission>,
}
