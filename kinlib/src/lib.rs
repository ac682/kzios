#![no_std]
#![warn(missing_docs)]
#![allow(unused)]

//! standard library for kzios

/// File system interaction
pub mod fs;
/// Process self and inter-process interaction
pub mod proc;
mod raw_call;