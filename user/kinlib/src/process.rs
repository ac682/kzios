/// ExitCode(i32) type for process
pub type ExitCode = i32;
/// Pid(u32) type for process
pub type Pid = u32;
/// Address(u64) type for process
pub type Address = u64;

/// The object could be converted to a exit code
pub trait Termination {
    /// To exit code and consume self
    fn to_exit_code(self) -> isize;
}

impl Termination for () {
    fn to_exit_code(self) -> isize {
        0
    }
}