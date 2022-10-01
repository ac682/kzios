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
