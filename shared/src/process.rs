use crate::ExitCode;

/// Process's main function product
pub trait Termination {
    /// Get completed process's exit code
    fn to_exit_code(self) -> ExitCode;
}

impl Termination for (){
    fn to_exit_code(self) -> ExitCode {
        0
    }
}