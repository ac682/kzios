use alloc::{borrow::ToOwned, boxed::Box, string::String, vec, vec::Vec};
use erhino_shared::{
    fal::{DentryAttribute, PropertyKind},
    time::Timestamp,
};
use flagset::FlagSet;

use crate::{call::sys_read, ipc::tunnel::Runnel};

use super::{DentryValue, FileSystemError};

macro_rules! dentry_method {
    ($method: ident, $result: ty) => {
        pub fn $method(&self) -> $result {
            match self {
                Dentry::Directory(directory) => directory.$method(),
                Dentry::Link(link) => link.$method(),
                Dentry::MountPoint(mountpoint) => mountpoint.$method(),
                Dentry::Property(property) => property.$method(),
                Dentry::Stream(stream) => stream.$method(),
            }
        }
    };
}

macro_rules! dentry_sub_methods {
    () => {
        pub fn name(&self) -> &str {
            &self.name
        }

        pub fn fullname(&self) -> &str {
            &self.fullname
        }

        pub fn created_at(&self) -> Timestamp {
            self.created_at
        }

        pub fn modified_at(&self) -> Timestamp {
            self.modified_at
        }

        pub fn r#move(&mut self, to: &str) -> Result<(), FileSystemError> {
            super::r#move(&self.fullname, to)?;
            self.fullname = to.to_owned();
            Ok(())
        }

        pub fn delete(self) -> Result<(), FileSystemError> {
            super::delete(&self.fullname)
        }
    };
}

pub enum Dentry {
    Directory(Directory),
    Link(Link),
    Stream(Stream),
    Property(Property),
    MountPoint(MountPoint),
}

impl Dentry {
    dentry_method!(name, &str);

    dentry_method!(fullname, &str);

    dentry_method!(created_at, Timestamp);

    dentry_method!(modified_at, Timestamp);
}

pub struct Directory {
    name: String,
    fullname: String,
    created_at: Timestamp,
    modified_at: Timestamp,
    attributes: FlagSet<DentryAttribute>,
    children: Vec<Dentry>,
}

impl Directory {
    pub(crate) fn new(
        name: &str,
        fullname: &str,
        created: Timestamp,
        modified: Timestamp,
        attr: FlagSet<DentryAttribute>,
        children: Vec<Dentry>,
    ) -> Self {
        Self {
            name: name.to_owned(),
            fullname: fullname.to_owned(),
            created_at: created,
            modified_at: modified,
            attributes: attr,
            children,
        }
    }

    dentry_sub_methods!();

    pub fn attributes(&self) -> &FlagSet<DentryAttribute> {
        &self.attributes
    }

    pub fn children(&self) -> &[Dentry] {
        &self.children
    }
}

pub struct Link {
    name: String,
    fullname: String,
    created_at: Timestamp,
    modified_at: Timestamp,
    attributes: FlagSet<DentryAttribute>,
}

impl Link {
    pub(crate) fn new(
        name: &str,
        fullname: &str,
        created: Timestamp,
        modified: Timestamp,
        attr: FlagSet<DentryAttribute>,
    ) -> Self {
        Self {
            name: name.to_owned(),
            fullname: fullname.to_owned(),
            created_at: created,
            modified_at: modified,
            attributes: attr,
        }
    }

    dentry_sub_methods!();

    pub fn attributes(&self) -> &FlagSet<DentryAttribute> {
        &self.attributes
    }
}

pub struct Stream {
    name: String,
    fullname: String,
    created_at: Timestamp,
    modified_at: Timestamp,
    size: usize,
    attributes: FlagSet<DentryAttribute>,
}

impl Stream {
    pub(crate) fn new(
        name: &str,
        fullname: &str,
        created: Timestamp,
        modified: Timestamp,
        size: usize,
        attr: FlagSet<DentryAttribute>,
    ) -> Self {
        Self {
            name: name.to_owned(),
            fullname: fullname.to_owned(),
            created_at: created,
            modified_at: modified,
            size,
            attributes: attr,
        }
    }

    dentry_sub_methods!();

    pub fn size(&self) -> usize {
        self.size
    }

    pub fn attributes(&self) -> &FlagSet<DentryAttribute> {
        &self.attributes
    }

    pub fn open(&self) -> Result<Runnel, FileSystemError> {
        todo!()
    }
}

pub struct Property {
    name: String,
    fullname: String,
    created_at: Timestamp,
    modified_at: Timestamp,
    size: usize,
    attributes: FlagSet<DentryAttribute>,
    kind: PropertyKind,
}

impl Property {
    pub(crate) fn new(
        name: &str,
        fullname: &str,
        created: Timestamp,
        modified: Timestamp,
        size: usize,
        attr: FlagSet<DentryAttribute>,
        kind: PropertyKind,
    ) -> Self {
        Self {
            name: name.to_owned(),
            fullname: fullname.to_owned(),
            created_at: created,
            modified_at: modified,
            size,
            attributes: attr,
            kind,
        }
    }

    dentry_sub_methods!();

    pub fn size(&self) -> usize {
        self.size
    }

    pub fn attributes(&self) -> &FlagSet<DentryAttribute> {
        &self.attributes
    }

    pub fn kind(&self) -> PropertyKind {
        self.kind
    }

    pub fn read(&self) -> Result<DentryValue, FileSystemError> {
        let mut buffer = vec![0u8; self.size];
        unsafe {
            match sys_read(&self.fullname, &mut buffer) {
                Ok(read) => {
                    buffer.truncate(read);
                    match self.kind {
                        PropertyKind::Integer => {
                            if read == 8 {
                                Ok(DentryValue::Integer(i64::from_ne_bytes([
                                    buffer[0], buffer[1], buffer[2], buffer[3], buffer[4],
                                    buffer[5], buffer[6], buffer[7],
                                ])))
                            } else {
                                Err(FileSystemError::SystemError)
                            }
                        }
                        PropertyKind::Integers => {
                            if read % 8 == 0 {
                                let count = read / 8;
                                let mut ints = Vec::<i64>::with_capacity(count);
                                for i in 0..count {
                                    let int = i64::from_ne_bytes([
                                        buffer[i * 8 + 0],
                                        buffer[i * 8 + 1],
                                        buffer[i * 8 + 2],
                                        buffer[i * 8 + 3],
                                        buffer[i * 8 + 4],
                                        buffer[i * 8 + 5],
                                        buffer[i * 8 + 6],
                                        buffer[i * 8 + 7],
                                    ]);
                                    ints.push(int);
                                }
                                ints.truncate(count);
                                Ok(DentryValue::Integers(ints))
                            } else {
                                Err(FileSystemError::SystemError)
                            }
                        }
                        PropertyKind::Decimal => {
                            if read == 8 {
                                Ok(DentryValue::Decimal(f64::from_ne_bytes([
                                    buffer[0], buffer[1], buffer[2], buffer[3], buffer[4],
                                    buffer[5], buffer[6], buffer[7],
                                ])))
                            } else {
                                Err(FileSystemError::SystemError)
                            }
                        }
                        PropertyKind::Decimals => {
                            if read % 8 == 0 {
                                let count = read / 8;
                                let mut decs = Vec::<f64>::with_capacity(count);
                                for i in 0..count {
                                    let int = f64::from_ne_bytes([
                                        buffer[i * 8 + 0],
                                        buffer[i * 8 + 1],
                                        buffer[i * 8 + 2],
                                        buffer[i * 8 + 3],
                                        buffer[i * 8 + 4],
                                        buffer[i * 8 + 5],
                                        buffer[i * 8 + 6],
                                        buffer[i * 8 + 7],
                                    ]);
                                    decs.push(int);
                                }
                                decs.truncate(count);
                                Ok(DentryValue::Decimals(decs))
                            } else {
                                Err(FileSystemError::SystemError)
                            }
                        }
                        PropertyKind::String => {
                            if let Ok(s) = String::from_utf8(buffer) {
                                Ok(DentryValue::String(s))
                            } else {
                                Err(FileSystemError::SystemError)
                            }
                        }
                        PropertyKind::Blob => Ok(DentryValue::Blob(buffer)),
                    }
                }
                Err(err) => Err(FileSystemError::from(err)),
            }
        }
    }
}

pub struct MountPoint {
    name: String,
    fullname: String,
    created_at: Timestamp,
    modified_at: Timestamp,
    mounted: Option<Box<Dentry>>,
}

impl MountPoint {
    pub(crate) fn new(
        name: &str,
        fullname: &str,
        created: Timestamp,
        modified: Timestamp,
        mounted: Option<Dentry>,
    ) -> Self {
        Self {
            name: name.to_owned(),
            fullname: fullname.to_owned(),
            created_at: created,
            modified_at: modified,
            mounted: mounted.map(|d| Box::new(d)),
        }
    }

    dentry_sub_methods!();

    pub fn mounted(&self) -> Option<&Dentry> {
        self.mounted.as_ref().map(|d| d.as_ref())
    }
}
