use erhino_shared::path::Path;

use erhino_shared::fal::{Dentry, DentryAttribute, FileSystem, FileAbstractLayerError};

// 结构
// 挂载到 rootfs 的 /proc
// (/proc)/{pid}/{prop}
// (/proc)/{pid}/usage/{prop}

pub struct Procfs {}

impl Procfs {
    pub fn new() -> Self {
        Self{}
    }
}

impl FileSystem for Procfs {
    fn is_property_supported(&self) -> bool {
        true
    }

    fn is_stream_supported(&self) -> bool {
        false
    }

    fn lookup(&self, path: Path) -> Result<&dyn Dentry, FileAbstractLayerError> {
        todo!()
    }
}
