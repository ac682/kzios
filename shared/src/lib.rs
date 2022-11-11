#![no_std]
#![warn(missing_docs)]

//! # eRhino shared lib
//!
//! Predefined types and system calls

extern crate alloc;

/// System calls
pub mod call;
/// Process types
pub mod proc;
/// Memory related
pub mod mem;
/// Service
pub mod service;