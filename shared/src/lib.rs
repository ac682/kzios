#![no_std]
#![warn(missing_docs)]

//! # eRhino shared lib
//!
//! Predefined types and system calls

/// System calls
pub mod call;
/// Process types
pub mod proc;
/// Memory related
pub mod mem;