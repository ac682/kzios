use core::mem::size_of;

use alloc::borrow::ToOwned;
use alloc::string::{String, ToString};
use alloc::vec;
use alloc::vec::Vec;
use erhino_shared::path::{Component, Path};

use erhino_shared::fal::{
    Dentry, DentryAttribute, DentryMeta, DentryType, FileKind, FileSystem,
    FilesystemAbstractLayerError, PropertyKind,
};
use erhino_shared::proc::Pid;
use flagset::FlagSet;

use crate::debug;
use crate::hart::SchedulerImpl;
use crate::task::sched::Scheduler;

enum FsLayer {
    Root,
    Proc(Pid),
    Memory(Pid, Option<String>),
}

// 结构
// 挂载到 rootfs 的 /proc
// (/proc)/{pid}/{prop}
// (/proc)/{pid}/memory/{prop}
// (/proc)/{pid}/traits/{trait}

pub struct Procfs {}

impl Procfs {
    pub fn new() -> Self {
        Self {}
    }

    fn parse(path: Path) -> Result<FsLayer, FilesystemAbstractLayerError> {
        if path.is_absolute() {
            let mut iter = path.iter();
            if let Some(Component::Root) = iter.next() {
                if let Some(Component::Normal(pid)) = iter.next() {
                    if let Ok(id) = pid.parse::<Pid>() {
                        if let Some(Component::Normal(prop)) = iter.next() {
                            match prop {
                                "memory" => {
                                    if let Some(Component::Normal(field)) = iter.next() {
                                        if let None = iter.next() {
                                            Ok(FsLayer::Memory(id, Some(field.to_owned())))
                                        } else {
                                            Err(FilesystemAbstractLayerError::NotFound)
                                        }
                                    } else {
                                        Ok(FsLayer::Memory(id, None))
                                    }
                                }
                                _ => Err(FilesystemAbstractLayerError::NotFound),
                            }
                        } else {
                            Ok(FsLayer::Proc(id))
                        }
                    } else {
                        Err(FilesystemAbstractLayerError::NotFound)
                    }
                } else {
                    Ok(FsLayer::Root)
                }
            } else {
                Err(FilesystemAbstractLayerError::InvalidPath)
            }
        } else {
            Err(FilesystemAbstractLayerError::InvalidPath)
        }
    }

    fn read_prop(pid: Pid, prop: &str) -> Result<Vec<u8>, FilesystemAbstractLayerError> {
        let mut buffer: Option<Vec<u8>> = None;
        SchedulerImpl::find(pid, |p| match prop {
            "page" => buffer = Some((p.usage.page as i64).to_ne_bytes().to_vec()),
            "program" => buffer = Some((p.usage.program as i64).to_ne_bytes().to_vec()),
            "heap" => buffer = Some((p.usage.heap as i64).to_ne_bytes().to_vec()),
            "stack" => buffer = Some((p.usage.stack as i64).to_ne_bytes().to_vec()),
            _ => {}
        });
        if let Some(res) = buffer {
            Ok(res)
        } else {
            Err(FilesystemAbstractLayerError::NotFound)
        }
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

    fn spawn_proc(pid: Pid) -> Dentry {
        let props = vec![Dentry::new(
            "memory".to_owned(),
            0,
            0,
            0,
            DentryAttribute::Executable | DentryAttribute::Readable,
            DentryMeta::Directory(Vec::new()),
        )];
        Dentry::new(
            pid.to_string(),
            0,
            0,
            0,
            DentryAttribute::Readable | DentryAttribute::Executable,
            DentryMeta::Directory(props),
        )
    }

    fn spawn_usage() -> Dentry {
        let props = vec![
            Dentry::new(
                "page".to_owned(),
                0,
                0,
                size_of::<i64>(),
                DentryAttribute::Readable.into(),
                DentryMeta::File(FileKind::Property(PropertyKind::Integer)),
            ),
            Dentry::new(
                "program".to_owned(),
                0,
                0,
                size_of::<i64>(),
                DentryAttribute::Readable.into(),
                DentryMeta::File(FileKind::Property(PropertyKind::Integer)),
            ),
            Dentry::new(
                "heap".to_owned(),
                0,
                0,
                size_of::<i64>(),
                DentryAttribute::Readable.into(),
                DentryMeta::File(FileKind::Property(PropertyKind::Integer)),
            ),
            Dentry::new(
                "stack".to_owned(),
                0,
                0,
                size_of::<i64>(),
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

    fn spawn_usage_field(field: &str) -> Dentry {
        match field {
            "page" => Dentry::new(
                "page".to_owned(),
                0,
                0,
                size_of::<i64>(),
                DentryAttribute::Readable.into(),
                DentryMeta::File(FileKind::Property(PropertyKind::Integer)),
            ),
            "program" => Dentry::new(
                "program".to_owned(),
                0,
                0,
                size_of::<i64>(),
                DentryAttribute::Readable.into(),
                DentryMeta::File(FileKind::Property(PropertyKind::Integer)),
            ),
            "heap" => Dentry::new(
                "heap".to_owned(),
                0,
                0,
                size_of::<i64>(),
                DentryAttribute::Readable.into(),
                DentryMeta::File(FileKind::Property(PropertyKind::Integer)),
            ),
            "stack" => Dentry::new(
                "stack".to_owned(),
                0,
                0,
                size_of::<i64>(),
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
        if let Ok(layer) = Self::parse(path) {
            match layer {
                FsLayer::Root => Ok(Self::spawn_root()),
                FsLayer::Proc(pid) => Ok(Self::spawn_proc(pid)),
                FsLayer::Memory(_, option) => {
                    if let Some(prop) = option {
                        Ok(Self::spawn_usage_field(prop.as_str()))
                    } else {
                        Ok(Self::spawn_usage())
                    }
                }
            }
        } else {
            Err(FilesystemAbstractLayerError::InvalidPath)
        }
    }

    fn create(
        &self,
        _path: Path,
        _kind: DentryType,
        _attr: FlagSet<DentryAttribute>,
    ) -> Result<(), FilesystemAbstractLayerError> {
        Err(FilesystemAbstractLayerError::Unsupported)
    }

    fn read(&self, path: Path, length: usize) -> Result<Vec<u8>, FilesystemAbstractLayerError> {
        debug!("procfs.read {}", path);
        if let Ok(layer) = Self::parse(path) {
            match layer {
                FsLayer::Root => Err(FilesystemAbstractLayerError::Unsupported),
                FsLayer::Proc(_) => Err(FilesystemAbstractLayerError::Unsupported),
                FsLayer::Memory(pid, option) => {
                    if let Some(prop) = option {
                        Self::read_prop(pid, prop.as_str())
                    } else {
                        Err(FilesystemAbstractLayerError::Unsupported)
                    }
                }
            }
        } else {
            Err(FilesystemAbstractLayerError::InvalidPath)
        }
    }

    fn write(&self, _path: Path, _value: &[u8]) -> Result<(), FilesystemAbstractLayerError> {
        Err(FilesystemAbstractLayerError::Unsupported)
    }
}
