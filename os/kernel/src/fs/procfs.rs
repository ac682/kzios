use erhino_shared::path::Path;

use erhino_shared::fal::{Dentry, FileSystem, FileSystemError, DentryAttribute};

// 结构
// 挂载到 rootfs 的 /proc
// (/proc)/{pid}/{prop}
// (/proc)/{pid}/usage/{prop}

pub struct Procfs {}

impl FileSystem for Procfs{
    fn is_property_supported(&self) -> bool {
        true
    }

    fn is_stream_supported(&self) -> bool {
        false
    }

    fn lookup(&self, path: Path) -> Result<&dyn Dentry, FileSystemError> {
        todo!()
    }

    fn make_dir<A: Into<flagset::FlagSet<DentryAttribute>> + Copy>(
        &mut self,
        path: Path,
        attr: A,
    ) -> Result<&dyn Dentry, FileSystemError> {
        todo!()
    }
}