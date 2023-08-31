// 定义一套接口和元数据，不包含实际数据

use alloc::string::String;
use erhino_shared::path::Path;
use flagset::{flags, FlagSet};

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
    kind: DentryKind,
    attributes: FlagSet<DentryAttribute>,
}

pub enum DentryKind {
    Directory,
    Link,
    File,
    MountPoint(),
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

pub trait FileSystem {
    fn is_property_supported(&self) -> bool;
    fn is_stream_supported(&self) -> bool;
    fn is_directory_supported(&self) -> bool;
    fn find_entry(path: Path) -> Option<Dentry>;
}
