/// ExitCode(i64) type for process
pub type ExitCode = isize;
/// Pid(u32) type for process
pub type Pid = u32;
/// Tid(u32) type for thread
/// If uniform thread-id required, It is uni_tid = ((pid << 32) + tid)
pub type Tid = u32;
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


#[derive(Debug, PartialEq)]
/// States of process
pub enum ProcessState {
    /// Can be picked as running process
    Ready,
    /// Code is being executed
    Running,
    /// Waiting for some signal and need to be waked up
    Sleeping,
    /// Finished, process would be cleaned up and pid put into recycling
    Dead,
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
