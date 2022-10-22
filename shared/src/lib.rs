#![no_std]
#![warn(missing_docs)]

//! # eRhino shared lib
//!
//! Predefined types and system calls

/// System calls
pub mod call;
/// Process types
pub mod process;
/// Memory paging stuff
pub mod page;

/// ExitCode(i64) type for process
pub type ExitCode = i64;
/// Pid(u32) type for process
pub type Pid = u32;
/// Tid(u32) type for thread
/// If uniform thread-id required, It is uni_tid = ((pid << 32) + tid)
pub type Tid = u32;
/// Address(u64) type for process
pub type Address = usize;
/// PageNumber(u64) for process
pub type PageNumber = usize;
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
