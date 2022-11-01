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

/// Predefined signal numbers
#[repr(u64)]
#[derive(FromPrimitive, ToPrimitive)]
pub enum Signal {
    /// Do nothing
    Nop = 0b1,
    /// Interrupt current workflow but not quit
    Interrupt = 0b10,
    /// Request to finalize the job and quit
    Terminate = 0b100,
    /// Kill the process after signal handled within a period of time
    Stop = 0b1000
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
pub enum WaitingReason{
    /// Waken up when time up
    Timer,
    /// Sending message blocks itself
    SendBusy,
    /// Receiving message blocks itself
    ReceiveBusy
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
