#![no_std]

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
/// Locks
pub mod sync;
/// Filesystem abstract layer
pub mod fal;
/// eRhino path string utilities
pub mod path;
