#![feature(lang_items, panic_info_message, alloc_error_handler)]
// Don't link to std. We are std.
#![no_std]

pub use erhino_shared::*;

pub mod call;
pub mod signal;
mod rt;
