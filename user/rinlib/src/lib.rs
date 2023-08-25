#![feature(lang_items, panic_info_message, alloc_error_handler)]
// Don't link to std. We are std.
#![no_std]
#![allow(internal_features)]

pub use erhino_shared as shared;

extern crate alloc;

mod call;
pub mod dbg;
pub mod ipc;
pub mod mm;
pub mod preclude;
mod rt;
pub mod thread;
