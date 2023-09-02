// 定义一套接口和元数据，不包含实际数据

use alloc::string::String;
use flagset::{flags, FlagSet};
use path::Path;

use crate::{path, time::Timestamp};

flags! {
    pub enum DentryAttribute: u8{
        None = 0,
        Readable = 1 << 0,
        Writeable = 1 << 1,
        Executable = 1 << 2
    }
}

pub trait Dentry {
    fn name(&self) -> &str;
    fn attributes(&self) -> &FlagSet<DentryAttribute>;
    fn kind(&self) -> &DentryKind;
}

pub enum DentryKind {
    Directory,
    Link,
    File(File),
    MountPoint,
}

#[repr(u8)]
pub enum DentryType{
    Directory = 0,
    Link,
    Stream,
    Property,
    MountPoint
}

/// Structure for serialization
pub struct DentryMeta{
    kind: DentryType,
    attr: FlagSet<DentryAttribute>,
    created_at: Timestamp,
    modified_at: Timestamp,
    size: usize,
    in_use: bool,
    name_length: usize,
    has_next: bool,
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
pub enum FileSystemError {
    InvalidPath,
    NotFound,
}

pub trait FileSystem {
    fn is_property_supported(&self) -> bool;
    fn is_stream_supported(&self) -> bool;
    fn lookup(&self, path: Path) -> Result<&dyn Dentry, FileSystemError>;
    fn make_dir<A: Into<FlagSet<DentryAttribute>> + Copy>(
        &mut self,
        path: Path,
        attr: A,
    ) -> Result<&dyn Dentry, FileSystemError>;
}
