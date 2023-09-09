// 定义一套接口和元数据，不包含实际数据

use core::fmt::Display;

use alloc::{string::String, vec::Vec};
use flagset::{flags, FlagSet};
use path::Path;

use crate::{path, proc::Pid, time::Timestamp};

/// Mountpoint id, may be pid or internal id
pub type Mid = u64;

flags! {
    pub enum DentryAttribute: u8{
        None = 0,
        Readable = 1 << 0,
        Writeable = 1 << 1,
        Executable = 1 << 2
    }
}

pub struct Dentry {
    name: String,
    attr: FlagSet<DentryAttribute>,
    meta: DentryMeta,
}

impl Dentry {
    pub fn new(name: String, attr: FlagSet<DentryAttribute>, meta: DentryMeta) -> Self {
        Self { name, attr, meta }
    }
    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn attributes(&self) -> &FlagSet<DentryAttribute> {
        &self.attr
    }

    pub fn meta(&self) -> &DentryMeta {
        &self.meta
    }
}

pub enum DentryMeta {
    Directory(Vec<Dentry>),
    Link,
    File(File),
    MountPoint(Mid),
}

#[repr(u8)]
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum DentryType {
    Directory = 0,
    Link,
    Stream,
    Property,
    MountPoint,
}

impl Display for DentryType {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "{:?}", self)
    }
}

impl From<&DentryMeta> for DentryType {
    fn from(value: &DentryMeta) -> Self {
        match &value {
            DentryMeta::Link => DentryType::Link,
            DentryMeta::File(file) => match file {
                File::Stream => DentryType::Stream,
                File::Property(_) => DentryType::Property,
            },
            DentryMeta::MountPoint(_) => DentryType::MountPoint,
            DentryMeta::Directory(_) => DentryType::Directory,
        }
    }
}

/// Structure for serialization
#[repr(C)]
pub struct DentryObject {
    pub kind: DentryType,
    pub attr: u8,
    pub created_at: Timestamp,
    pub modified_at: Timestamp,
    pub size: usize,
    pub in_use: bool,
    pub name_length: usize,
}

impl DentryObject {
    pub fn new(
        kind: DentryType,
        attr: &FlagSet<DentryAttribute>,
        created: Timestamp,
        modified: Timestamp,
        size: usize,
        in_use: bool,
        name_length: usize,
    ) -> Self {
        Self {
            kind,
            attr: attr.bits(),
            created_at: created,
            modified_at: modified,
            size,
            in_use,
            name_length,
        }
    }
}

pub enum File {
    Stream,
    Property(PropertyKind),
}

pub enum PropertyKind {
    Integer,
    Integers,
    Decimal,
    Decimals,
    String,
    Blob,
}

#[derive(Debug)]
pub enum FilesystemAbstractLayerError {
    InvalidPath,
    NotFound,
    NotAccessible,
    Mistyped,
    Conflict,
    ForeignMountPoint(Path, Mid),
}

pub trait FileSystem {
    fn is_property_supported(&self) -> bool;
    fn is_stream_supported(&self) -> bool;
    fn lookup(&self, path: Path) -> Result<Dentry, FilesystemAbstractLayerError>;
}
