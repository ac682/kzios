use alloc::vec::Vec;
use erhino_shared::path::Path;

use crate::fal::{FileSystem, Dentry};

use super::local::LocalDentry;

pub struct Rootfs {
    entries: Vec<LocalDentry>
}

impl FileSystem for Rootfs {
    fn is_property_supported(&self) -> bool {
        false
    }

    fn is_stream_supported(&self) -> bool {
        false
    }

    fn is_directory_supported(&self) -> bool {
        true
    }

    fn find_entry(path: Path) -> Option<Dentry> {
        todo!()
    }
}
