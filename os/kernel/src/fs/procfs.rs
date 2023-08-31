use erhino_shared::path::Path;

use crate::fal::{Dentry, FileSystem};

pub struct Procfs {}

impl FileSystem for Procfs {
    fn is_property_supported(&self) -> bool {
        true
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
