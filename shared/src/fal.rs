// 定义一套接口和元数据，不包含实际数据

use core::fmt::Display;

use alloc::{string::String, vec::Vec};
use flagset::{flags, FlagSet};
use num_derive::{FromPrimitive, ToPrimitive};
use path::Path;

use crate::{path, proc::Pid, time::Timestamp};

/// Mountpoint id, may be pid or internal id
pub type Mid = u64;

flags! {
    pub enum DentryAttribute: u8{
        None = 0,
        Readable = 1 << 0,
        Writeable = 1 << 1,
        Executable = 1 << 2,
        PrivilegedReadable = 1 << 3,
        PrivilegedWriteable = 1 << 4,
        PrivilegedExecutable = 1 << 5
    }
}

pub struct Dentry {
    name: String,
    created: Timestamp,
    modified: Timestamp,
    size: usize,
    attr: FlagSet<DentryAttribute>,
    meta: DentryMeta,
}

impl Dentry {
    pub fn new(
        name: String,
        created: Timestamp,
        modified: Timestamp,
        size: usize,
        attr: FlagSet<DentryAttribute>,
        meta: DentryMeta,
    ) -> Self {
        Self {
            name,
            created,
            modified,
            size,
            attr,
            meta,
        }
    }

    pub fn created_at(&self) -> Timestamp {
        self.created
    }

    pub fn modified_at(&self) -> Timestamp {
        self.modified
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn size(&self) -> usize {
        self.size
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
    File(FileKind),
    MountPoint(Mid),
}

#[repr(u8)]
#[derive(Clone, Copy, Debug, PartialEq, FromPrimitive, ToPrimitive)]
pub enum DentryType {
    Directory = 0,
    Link,
    Stream,
    Integer,
    Integers,
    Decimal,
    Decimals,
    String,
    Blob,
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
            DentryMeta::File(FileKind::Stream) => DentryType::Stream,
            DentryMeta::File(FileKind::Property(PropertyKind::Integer)) => DentryType::Integer,
            DentryMeta::File(FileKind::Property(PropertyKind::Integers)) => DentryType::Integers,
            DentryMeta::File(FileKind::Property(PropertyKind::Decimal)) => DentryType::Decimal,
            DentryMeta::File(FileKind::Property(PropertyKind::Decimals)) => DentryType::Decimals,
            DentryMeta::File(FileKind::Property(PropertyKind::String)) => DentryType::Stream,
            DentryMeta::File(FileKind::Property(PropertyKind::Blob)) => DentryType::Blob,
            DentryMeta::MountPoint(_) => DentryType::MountPoint,
            DentryMeta::Directory(_) => DentryType::Directory,
        }
    }
}

impl From<&PropertyKind> for DentryType {
    fn from(value: &PropertyKind) -> Self {
        match value {
            PropertyKind::Integer => DentryType::Integer,
            PropertyKind::Integers => DentryType::Integers,
            PropertyKind::Decimal => DentryType::Decimal,
            PropertyKind::Decimals => DentryType::Decimals,
            PropertyKind::String => DentryType::String,
            PropertyKind::Blob => DentryType::Blob,
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
    pub size: u64,
    pub in_use: bool,
    pub name_length: u64,
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
            size: size as u64,
            in_use,
            name_length: name_length as u64,
        }
    }
}

pub enum FileKind {
    Stream,
    Property(PropertyKind),
}

#[derive(Debug, Clone, Copy)]
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
    Unsupported,
    ForeignMountPoint(Path, Mid),
}

pub trait FileSystem {
    fn is_property_supported(&self) -> bool;
    fn is_stream_supported(&self) -> bool;
    fn lookup(&self, path: Path) -> Result<Dentry, FilesystemAbstractLayerError>;
    fn create(
        &self,
        path: Path,
        kind: DentryType,
        attr: FlagSet<DentryAttribute>,
    ) -> Result<(), FilesystemAbstractLayerError>;
    fn read(&self, path: Path) -> Result<Vec<u8>, FilesystemAbstractLayerError>;
}
