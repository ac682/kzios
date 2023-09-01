use core::fmt::{Display, Write};
use core::sync::atomic::AtomicUsize;

use alloc::borrow::ToOwned;
use alloc::string::String;
use alloc::vec::Vec;
use erhino_shared::path::{Component, Path, PathIterator};

use erhino_shared::fal::{Dentry, DentryAttribute, FileSystem, FileSystemError, DentryKind};
use flagset::FlagSet;

pub enum Registry{
    Local,
    Remote
}

#[derive(Debug)]
pub enum LocalDentryError {
    InvalidFilename,
}

pub struct LocalDentry {
    kind: LocalDentryKind,
    name: String,
    attributes: FlagSet<DentryAttribute>,
    ref_count: AtomicUsize,
}

impl LocalDentry {
    pub fn new_dir<A: Into<FlagSet<DentryAttribute>>>(
        name: &str,
        attr: A,
    ) -> Result<Self, LocalDentryError> {
        if Path::is_filename(name) {
            Ok(Self {
                kind: LocalDentryKind::Directory(Vec::new()),
                name: name.to_owned(),
                attributes: attr.into(),
                ref_count: AtomicUsize::new(0),
            })
        } else {
            Err(LocalDentryError::InvalidFilename)
        }
    }

    pub fn real(&self) -> &LocalDentryKind {
        &self.kind
    }

    pub fn real_mut(&mut self) -> &mut LocalDentryKind {
        &mut self.kind
    }
}

impl Dentry for LocalDentry {
    fn name(&self) -> &str {
        &self.name
    }

    fn attributes(&self) -> &FlagSet<DentryAttribute> {
        &self.attributes
    }

    fn kind(&self) -> &DentryKind {
        todo!()
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
            LocalDentryKind::Link(path) => writeln!(f, "{} -> {}", self.name, path),
            LocalDentryKind::MountPoint(_) => writeln!(f, "({})", self.name),
        }
    }
}

pub enum LocalDentryKind {
    Directory(Vec<LocalDentry>),
    Link(Path),
    MountPoint(Registry),
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

pub struct Rootfs {
    root: LocalDentry,
}

impl Rootfs {
    pub fn new() -> Self {
        Self {
            root: LocalDentry::new_dir("", DentryAttribute::Executable).unwrap(),
        }
    }

    fn find_entry<'a>(
        container: &'a LocalDentry,
        mut path: PathIterator,
    ) -> Result<&'a LocalDentry, FileSystemError> {
        if let Some(next) = path.next() {
            match next {
                Component::Normal(name) => {
                    if let LocalDentryKind::Directory(subs) = container.real() {
                        for s in subs {
                            if s.name() == name {
                                return Self::find_entry(s, path);
                            }
                        }
                        Err(FileSystemError::NotFound)
                    } else {
                        Err(FileSystemError::NotFound)
                    }
                }
                _ => unreachable!(),
            }
        } else {
            Ok(container)
        }
    }

    fn find_entry_mut<'a, A: Into<FlagSet<DentryAttribute>> + Copy>(
        container: &'a mut LocalDentry,
        mut path: PathIterator,
        attr: Option<A>,
    ) -> Result<&'a mut LocalDentry, FileSystemError> {
        if let Some(next) = path.next() {
            match next {
                Component::Normal(name) => {
                    if let LocalDentryKind::Directory(subs) = container.real_mut() {
                        let mut found: Option<usize> = None;
                        for (i, s) in subs.iter_mut().enumerate() {
                            if s.name() == name {
                                found = Some(i);
                            }
                        }
                        if let Some(i) = found {
                            Self::find_entry_mut(&mut subs[i], path, attr)
                        } else {
                            if let Some(a) = attr {
                                let sub = LocalDentry::new_dir(name, a.into()).unwrap();
                                subs.push(sub);
                                Self::find_entry_mut(subs.last_mut().unwrap(), path, attr)
                            } else {
                                Err(FileSystemError::NotFound)
                            }
                        }
                    } else {
                        Err(FileSystemError::NotFound)
                    }
                }
                _ => unreachable!(),
            }
        } else {
            Ok(container)
        }
    }
}

impl FileSystem for Rootfs {
    fn is_property_supported(&self) -> bool {
        false
    }

    fn is_stream_supported(&self) -> bool {
        false
    }

    fn lookup(&self, path: Path) -> Result<&dyn Dentry, FileSystemError> {
        if path.is_absolute() {
            let qualified = path.qualify().unwrap();
            let mut iter = qualified.iter();
            iter.next();
            Self::find_entry(&self.root, iter).map(|d| d as &dyn Dentry)
        } else {
            Err(FileSystemError::InvalidPath)
        }
    }

    fn make_dir<A: Into<FlagSet<DentryAttribute>> + Copy>(
        &mut self,
        path: Path,
        attr: A,
    ) -> Result<&dyn Dentry, FileSystemError> {
        if path.is_absolute() {
            let qualified = path.qualify().unwrap();
            let mut iter = qualified.iter();
            iter.next();
            Self::find_entry_mut(&mut self.root, iter, Some(attr)).map(|d| d as &dyn Dentry)
        } else {
            Err(FileSystemError::InvalidPath)
        }
    }
}

impl Display for Rootfs {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "{}", self.root)
    }
}
