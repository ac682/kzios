#![no_std]
#![warn(missing_docs)]

//! # eRhino shared lib
//!
//! Predefined types and system calls

extern crate alloc;

/// System calls
pub mod call;
/// Memory related
pub mod mem;
/// Process types
pub mod proc;
/// Service
pub mod service;
