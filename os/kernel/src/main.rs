#![no_std]
#![feature(lang_items, alloc_error_handler, panic_info_message)]
#![allow(internal_features)]

use core::arch::global_asm;

use board::{
    device::{
        bus::{BusKind, SpiDeviceKind},
        peripheral::PeripheralKind,
    },
    Board,
};
use dtb_parser::{prop::PropertyValue, traits::FindPropertyValue};

extern crate alloc;

mod board;
mod console;
mod driver;
mod external;
mod hart;
mod mm;
mod rt;
mod sbi;
mod sync;
mod task;
mod timer;
mod trap;

global_asm!(include_str!("assembly.asm"));

fn main() {
    let board = board::this_board();
    info!("{}", board);
    println!("\x1b[0;32m=LINK^START=\x1b[0m");
    println!("\x1b[0;33m=SEE^YOU^NEXT^TIME=\x1b[0m");
    sbi::system_reset(0, 0).expect("system reset failure");
}

fn boot(board: &Board) {
    let tree = board.tree();
    if let Some(chosen) = tree.find_node("/chosen") {
        if let Some(PropertyValue::String(path)) = chosen.value("boot-device-path") {
            if let Some(chain) = tree.find_along_path(path) {
                let mut iter = chain.iter();
                // root
                iter.next();
                // soc
                iter.next();
                while let Some(node) = iter.next() {
                    
                }
            }
        }
    }
}
