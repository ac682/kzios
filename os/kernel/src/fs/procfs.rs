use alloc::borrow::ToOwned;
use alloc::string::ToString;
use alloc::vec;
use alloc::vec::Vec;
use erhino_shared::path::{Component, Path};

use erhino_shared::fal::{
    Dentry, DentryAttribute, DentryMeta, FileKind, FileSystem, FilesystemAbstractLayerError,
    PropertyKind,
};
use erhino_shared::proc::Pid;

use crate::debug;
use crate::hart::SchedulerImpl;
use crate::task::sched::Scheduler;

// 结构
// 挂载到 rootfs 的 /proc
// (/proc)/{pid}/{prop}
// (/proc)/{pid}/usage/{prop}

pub struct Procfs {}

impl Procfs {
    pub fn new() -> Self {
        Self {}
    }

    fn spawn_root() -> Dentry {
        let mut proc = Vec::<Dentry>::new();
        for i in SchedulerImpl::snapshot() {
            proc.push(Dentry::new(
                i.to_string(),
                0,
                0,
                0,
                DentryAttribute::Executable | DentryAttribute::Readable,
                DentryMeta::Directory(Vec::with_capacity(0)),
            ));
        }
        Dentry::new(
            "".to_owned(),
            0,
            0,
            0,
            DentryAttribute::Executable | DentryAttribute::Readable,
            DentryMeta::Directory(proc),
        )
    }

    fn spawn_proc(pid: Pid) -> Option<Dentry> {
        let props = vec![Dentry::new(
            "memory".to_owned(),
            0,
            0,
            0,
            DentryAttribute::Executable | DentryAttribute::Readable,
            DentryMeta::Directory(Vec::new()),
        )];
        Some(Dentry::new(
            pid.to_string(),
            0,
            0,
            0,
            DentryAttribute::Readable | DentryAttribute::Executable,
            DentryMeta::Directory(props),
        ))
    }

    fn spawn_usage() -> Dentry {
        let props = vec![
            Dentry::new(
                "page".to_owned(),
                0,
                0,
                0,
                DentryAttribute::Readable.into(),
                DentryMeta::File(FileKind::Property(PropertyKind::Integer)),
            ),
            Dentry::new(
                "program".to_owned(),
                0,
                0,
                0,
                DentryAttribute::Readable.into(),
                DentryMeta::File(FileKind::Property(PropertyKind::Integer)),
            ),
            Dentry::new(
                "heap".to_owned(),
                0,
                0,
                0,
                DentryAttribute::Readable.into(),
                DentryMeta::File(FileKind::Property(PropertyKind::Integer)),
            ),
            Dentry::new(
                "stack".to_owned(),
                0,
                0,
                0,
                DentryAttribute::Readable.into(),
                DentryMeta::File(FileKind::Property(PropertyKind::Integer)),
            ),
        ];
        Dentry::new(
            "memory".to_owned(),
            0,
            0,
            0,
            DentryAttribute::Executable | DentryAttribute::Readable,
            DentryMeta::Directory(props),
        )
    }

    fn spawn_usage_field(pid: Pid, field: &str) -> Dentry {
        match field {
            "page" => Dentry::new(
                "page".to_owned(),
                0,
                0,
                0,
                DentryAttribute::Readable.into(),
                DentryMeta::File(FileKind::Property(PropertyKind::Integer)),
            ),
            "program" => Dentry::new(
                "program".to_owned(),
                0,
                0,
                0,
                DentryAttribute::Readable.into(),
                DentryMeta::File(FileKind::Property(PropertyKind::Integer)),
            ),
            "heap" => Dentry::new(
                "heap".to_owned(),
                0,
                0,
                0,
                DentryAttribute::Readable.into(),
                DentryMeta::File(FileKind::Property(PropertyKind::Integer)),
            ),
            "stack" => Dentry::new(
                "stack".to_owned(),
                0,
                0,
                0,
                DentryAttribute::Readable.into(),
                DentryMeta::File(FileKind::Property(PropertyKind::Integer)),
            ),
            _ => unimplemented!(),
        }
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
        debug!("procfs.lookup {}", path.as_str());
        if path.is_absolute() {
            if let Ok(qualified) = path.qualify() {
                let mut iter = qualified.iter();
                if let Some(Component::Root) = iter.next() {
                    if let Some(Component::Normal(pid)) = iter.next() {
                        if let Ok(id) = pid.parse::<Pid>() {
                            if let Some(Component::Normal(prop)) = iter.next() {
                                match prop {
                                    "memory" => {
                                        if let Some(Component::Normal(field)) = iter.next() {
                                            if let None = iter.next() {
                                                Ok(Self::spawn_usage_field(id, field))
                                            } else {
                                                Err(FilesystemAbstractLayerError::NotFound)
                                            }
                                        } else {
                                            Ok(Self::spawn_usage())
                                        }
                                    }
                                    _ => Err(FilesystemAbstractLayerError::NotFound),
                                }
                            } else {
                                Ok(Self::spawn_proc(id).unwrap())
                            }
                        } else {
                            Err(FilesystemAbstractLayerError::NotFound)
                        }
                    } else {
                        Ok(Self::spawn_root())
                    }
                } else {
                    Err(FilesystemAbstractLayerError::InvalidPath)
                }
            } else {
                Err(FilesystemAbstractLayerError::InvalidPath)
            }
        } else {
            Err(FilesystemAbstractLayerError::InvalidPath)
        }
    }

    fn read(&self, path: Path, buffer: &[u8]) -> Result<usize, FilesystemAbstractLayerError> {
        debug!("procfs.read {}", path);
        todo!()
    }
}
