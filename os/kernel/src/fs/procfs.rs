use core::mem::size_of;

use alloc::borrow::ToOwned;
use alloc::string::ToString;
use alloc::vec;
use alloc::vec::Vec;
use erhino_shared::path::{Component, Path};

use erhino_shared::fal::{
    Dentry, DentryAttribute, DentryMeta, DentryType, FileKind, FileSystem,
    FilesystemAbstractLayerError, PropertyKind,
};
use erhino_shared::proc::Pid;
use flagset::FlagSet;

use crate::hart::SchedulerImpl;
use crate::task::sched::Scheduler;

enum FsLayer {
    Root,
    Proc(Pid),
    PropPid(Pid),
    Memory,
    MemoryPage(Pid),
    MemoryProgram(Pid),
    MemoryHeap(Pid),
    MemoryStack(Pid),
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
                                "Memory" => {
                                    if let Some(Component::Normal(field)) = iter.next() {
                                        if let None = iter.next() {
                                            match field {
                                                "Page" => Ok(FsLayer::MemoryPage(id)),
                                                "Program" => Ok(FsLayer::MemoryProgram(id)),
                                                "Heap" => Ok(FsLayer::MemoryHeap(id)),
                                                "Stack" => Ok(FsLayer::MemoryStack(id)),
                                                _ => Err(FilesystemAbstractLayerError::NotFound),
                                            }
                                        } else {
                                            Err(FilesystemAbstractLayerError::NotFound)
                                        }
                                    } else {
                                        Ok(FsLayer::Memory)
                                    }
                                }
                                "Pid" => {
                                    if let None = iter.next() {
                                        Ok(FsLayer::PropPid(id))
                                    } else {
                                        Err(FilesystemAbstractLayerError::NotFound)
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

    fn read_prop(pid: Pid, prop: FsLayer) -> Result<Vec<u8>, FilesystemAbstractLayerError> {
        let mut buffer: Option<Vec<u8>> = None;
        SchedulerImpl::find(pid, |p| match prop {
            FsLayer::MemoryPage(_) => buffer = Some((p.usage.page as i64).to_ne_bytes().to_vec()),
            FsLayer::MemoryProgram(_) => {
                buffer = Some((p.usage.program as i64).to_ne_bytes().to_vec())
            }
            FsLayer::MemoryHeap(_) => buffer = Some((p.usage.heap as i64).to_ne_bytes().to_vec()),
            FsLayer::MemoryStack(_) => buffer = Some((p.usage.stack as i64).to_ne_bytes().to_vec()),
            _ => {}
        });
        if let Some(res) = buffer {
            Ok(res)
        } else {
            Err(FilesystemAbstractLayerError::NotFound)
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
        if let Ok(layer) = Self::parse(path) {
            match layer {
                FsLayer::Root => {
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
                    Ok(Dentry::new(
                        "".to_owned(),
                        0,
                        0,
                        0,
                        DentryAttribute::Executable | DentryAttribute::Readable,
                        DentryMeta::Directory(proc),
                    ))
                }
                FsLayer::Proc(pid) => {
                    let props = vec![
                        Dentry::new(
                            "Memory".to_owned(),
                            0,
                            0,
                            0,
                            DentryAttribute::Executable | DentryAttribute::Readable,
                            DentryMeta::Directory(Vec::new()),
                        ),
                        Dentry::new(
                            "Pid".to_owned(),
                            0,
                            0,
                            size_of::<Pid>(),
                            DentryAttribute::Readable.into(),
                            DentryMeta::File(FileKind::Property(PropertyKind::Integer)),
                        ),
                    ];
                    Ok(Dentry::new(
                        pid.to_string(),
                        0,
                        0,
                        0,
                        DentryAttribute::Readable | DentryAttribute::Executable,
                        DentryMeta::Directory(props),
                    ))
                }
                FsLayer::PropPid(_) => Ok(Dentry::new(
                    "Pid".to_owned(),
                    0,
                    0,
                    size_of::<i64>(),
                    DentryAttribute::Readable.into(),
                    DentryMeta::File(FileKind::Property(PropertyKind::Integer)),
                )),
                FsLayer::Memory => {
                    let props = vec![
                        Dentry::new(
                            "Page".to_owned(),
                            0,
                            0,
                            size_of::<i64>(),
                            DentryAttribute::Readable.into(),
                            DentryMeta::File(FileKind::Property(PropertyKind::Integer)),
                        ),
                        Dentry::new(
                            "Program".to_owned(),
                            0,
                            0,
                            size_of::<i64>(),
                            DentryAttribute::Readable.into(),
                            DentryMeta::File(FileKind::Property(PropertyKind::Integer)),
                        ),
                        Dentry::new(
                            "Heap".to_owned(),
                            0,
                            0,
                            size_of::<i64>(),
                            DentryAttribute::Readable.into(),
                            DentryMeta::File(FileKind::Property(PropertyKind::Integer)),
                        ),
                        Dentry::new(
                            "Stack".to_owned(),
                            0,
                            0,
                            size_of::<i64>(),
                            DentryAttribute::Readable.into(),
                            DentryMeta::File(FileKind::Property(PropertyKind::Integer)),
                        ),
                    ];
                    Ok(Dentry::new(
                        "Memory".to_owned(),
                        0,
                        0,
                        0,
                        DentryAttribute::Executable | DentryAttribute::Readable,
                        DentryMeta::Directory(props),
                    ))
                }
                FsLayer::MemoryPage(_) => Ok(Dentry::new(
                    "Page".to_owned(),
                    0,
                    0,
                    size_of::<i64>(),
                    DentryAttribute::Readable.into(),
                    DentryMeta::File(FileKind::Property(PropertyKind::Integer)),
                )),
                FsLayer::MemoryProgram(_) => Ok(Dentry::new(
                    "Program".to_owned(),
                    0,
                    0,
                    size_of::<i64>(),
                    DentryAttribute::Readable.into(),
                    DentryMeta::File(FileKind::Property(PropertyKind::Integer)),
                )),
                FsLayer::MemoryHeap(_) => Ok(Dentry::new(
                    "Heap".to_owned(),
                    0,
                    0,
                    size_of::<i64>(),
                    DentryAttribute::Readable.into(),
                    DentryMeta::File(FileKind::Property(PropertyKind::Integer)),
                )),
                FsLayer::MemoryStack(_) => Ok(Dentry::new(
                    "Stack".to_owned(),
                    0,
                    0,
                    size_of::<i64>(),
                    DentryAttribute::Readable.into(),
                    DentryMeta::File(FileKind::Property(PropertyKind::Integer)),
                )),
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

    fn read(&self, path: Path, _: usize) -> Result<Vec<u8>, FilesystemAbstractLayerError> {
        if let Ok(layer) = Self::parse(path) {
            match layer {
                FsLayer::Root => Err(FilesystemAbstractLayerError::Unsupported),
                FsLayer::Proc(_) => Err(FilesystemAbstractLayerError::Unsupported),
                FsLayer::PropPid(pid) => Ok((pid as i64).to_ne_bytes().to_vec()),
                FsLayer::Memory => Err(FilesystemAbstractLayerError::Unsupported),
                FsLayer::MemoryPage(pid) => Self::read_prop(pid, layer),
                FsLayer::MemoryProgram(pid) => Self::read_prop(pid, layer),
                FsLayer::MemoryHeap(pid) => Self::read_prop(pid, layer),
                FsLayer::MemoryStack(pid) => Self::read_prop(pid, layer),
            }
        } else {
            Err(FilesystemAbstractLayerError::InvalidPath)
        }
    }

    fn write(&self, _path: Path, _value: &[u8]) -> Result<(), FilesystemAbstractLayerError> {
        Err(FilesystemAbstractLayerError::Unsupported)
    }
}
