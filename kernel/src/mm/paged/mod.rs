use self::page_table::PageTable;

mod address;
pub mod frame_allocator;
mod page_table;

pub fn init() {
    let kernel = PageTable::new(2);
    // map some initial addresses
}
