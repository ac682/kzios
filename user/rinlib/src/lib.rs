#![feature(lang_items, panic_info_message, alloc_error_handler)]
// Don't link to std. We are std.
#![no_std]

pub use erhino_shared as shared;

extern crate alloc;

mod call;
pub mod dbg;
pub mod mm;
pub mod thread;
pub mod preclude;
mod rt;
