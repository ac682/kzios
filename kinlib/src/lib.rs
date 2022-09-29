//! # kinlib, kzios is no a lib
//! standard library for kzios

#![feature(panic_info_message, lang_items, linkage, start, alloc_error_handler)]
// Don't link to std. We are std.
#![no_std]
#![warn(missing_docs)]

//extern crate alloc;

/// Just io
pub mod io;
mod lang_items;
/// Process definition and operations
pub mod process;
mod rt;
/// System calls
pub mod syscall;
