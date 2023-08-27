use flagset::flags;

/// Address(u64) type for process
pub type Address = usize;
/// PageNumber(u64) for process
pub type PageNumber = usize;

flags! {
    /// Basic memory operation permissions
    pub enum MemoryRegionAttribute: usize{
        /// No access
        None = 0b0,
        /// Readable
        Read = 0b1,
        /// Writeable
        Write = 0b10,
        /// Executable
        Execute = 0b100
    }
}

/// Basic memory operation
#[derive(Debug)]
pub enum MemoryOperation {
    /// Read
    Read,
    /// Write
    Write,
    /// Execute
    Execute,
}
