use core::fmt::{Display, Write};

use alloc::{vec::Vec, string::String, borrow::ToOwned};
use erhino_shared::path::Path;
use flagset::{flags, FlagSet};

pub enum LocalRegister {
    Local(Vec<LocalDentry>),
    Remote,
}

impl LocalRegister {
    pub fn new() -> Self {
        Self::Local(Vec::new())
    }
}

impl Display for LocalRegister {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            LocalRegister::Remote => writeln!(f, "Remote"),
            LocalRegister::Local(children) => {
                let mut buffer = String::new();
                for child in children {
                    write!(buffer, "{}", child)?;
                }
                for i in buffer.split('\n') {
                    if !i.is_empty() {
                        writeln!(f, "|  {}", i)?;
                    }
                }
                Ok(())
            }
        }
    }
}

#[derive(Debug)]
pub enum LocalDentryError {
    InvalidFilename,
}

flags! {
    pub enum LocalDentryAttribute: u8{
        None = 0,
        Readable = 1 << 0,
        Writeable = 1 << 1,
        Executable = 1 << 2
    }
}

pub struct LocalDentry {
    kind: LocalDentryKind,
    name: String,
    attributes: FlagSet<LocalDentryAttribute>,
}

impl LocalDentry {
    pub fn new<A: Into<FlagSet<LocalDentryAttribute>> + Copy>(
        name: &str,
        kind: LocalDentryKind,
        attr: A,
    ) -> Result<Self, LocalDentryError> {
        Ok(Self {
            kind,
            name: name.to_owned(),
            attributes: attr.into(),
        })
    }

    pub fn filename(&self) -> &str {
        &self.name
    }
}

impl Display for LocalDentry {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match &self.kind {
            LocalDentryKind::Directory(children) => {
                writeln!(f, "{}/", self.name)?;
                let mut buffer = String::new();
                for child in children {
                    write!(buffer, "{}", child)?;
                }
                for i in buffer.split('\n') {
                    if !i.is_empty() {
                        writeln!(f, "|  {}", i)?;
                    }
                }
                Ok(())
            }
            LocalDentryKind::File(file) => writeln!(f, "{}: {}", self.name, file),
            LocalDentryKind::Link(path) => writeln!(f, "{} -> {}", self.name, path),
            LocalDentryKind::MountPoint(_) => writeln!(f, "({})", self.name),
        }
    }
}

pub enum LocalDentryKind {
    Directory(Vec<LocalDentry>),
    Link(Path),
    File(LocalFile),
    MountPoint(LocalRegister),
}

pub enum LocalFile {
    Stream,
    Property(LocalPropertyKind),
}

pub enum LocalPropertyKind {
    Integer(i64),
    Integers(Vec<i64>),
    Decimal(f64),
    Decimals(Vec<f64>),
    String(String),
    Blob(Vec<u8>),
}

impl Display for LocalFile {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            LocalFile::Stream => write!(f, "STREAM"),
            LocalFile::Property(property) => match property {
                LocalPropertyKind::Decimal(it) => write!(f, "{}", it),
                LocalPropertyKind::Decimals(it) => write!(f, "{:?}", it),
                LocalPropertyKind::Integer(it) => write!(f, "{}", it),
                LocalPropertyKind::Integers(it) => write!(f, "{:?}", it),
                LocalPropertyKind::String(it) => write!(f, "{}", it),
                LocalPropertyKind::Blob(_) => write!(f, "BLOB"),
            },
        }
    }
}
