use alloc::borrow::ToOwned;
use alloc::string::{String, ToString};
use erhino_shared::path::Path;

use erhino_shared::fal::{
    Dentry, DentryAttribute, DentryMeta, FileSystem, FilesystemAbstractLayerError,
};
use erhino_shared::proc::Pid;
use flagset::FlagSet;

use crate::task::proc::Process;

// 结构
// 挂载到 rootfs 的 /proc
// (/proc)/{pid}/{prop}
// (/proc)/{pid}/usage/{prop}

pub struct Procfs {}

impl Procfs {
    pub fn new() -> Self {
        Self {}
    }
}

impl FileSystem for Procfs {
    fn is_property_supported(&self) -> bool {
        true
    }

    fn is_stream_supported(&self) -> bool {
        false
    }

    fn lookup(&self, path: Path) -> Result<Dentry, FilesystemAbstractLayerError> {
        if path.is_absolute() {
            if let Ok(qualified) = path.qualify() {
                if qualified.as_str() == "/" {
                    Ok(Dentry::new(
                        "".to_owned(),
                        DentryAttribute::Executable | DentryAttribute::Readable,
                        DentryMeta::Link,
                    ))
                } else {
                    todo!()
                }
            } else {
                Err(FilesystemAbstractLayerError::InvalidPath)
            }
        } else {
            Err(FilesystemAbstractLayerError::InvalidPath)
        }
    }
}
