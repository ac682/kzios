use crate::ExitCode;

#[derive(Debug)]
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
