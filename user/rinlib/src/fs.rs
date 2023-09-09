use core::mem::size_of;

use alloc::{borrow::ToOwned, string::String, vec, vec::Vec};
use erhino_shared::{
    call::SystemCallError,
    fal::{DentryAttribute, DentryObject, DentryType},
    time::Timestamp,
};
use flagset::FlagSet;

use crate::{
    call::{sys_access, sys_inspect},
    debug,
};

#[derive(Debug, Clone, Copy)]
pub enum FileSystemError {
    Unknown,
    InvalidPath,
    NotFound,
    NotAccessible,
    // Filesystem mountpoint does not exist
    NotAvailable,
}

impl From<SystemCallError> for FileSystemError {
    fn from(value: SystemCallError) -> Self {
        match value {
            SystemCallError::IllegalArgument => FileSystemError::InvalidPath,
            SystemCallError::ObjectNotAccessible => FileSystemError::NotAccessible,
            SystemCallError::ObjectNotFound => FileSystemError::NotFound,
            _ => FileSystemError::Unknown,
        }
    }
}

pub struct Dentry {
    name: String,
    kind: DentryType,
    attr: FlagSet<DentryAttribute>,
    created_at: Timestamp,
    modified_at: Timestamp,
    children: Option<Vec<Dentry>>,
}

impl Dentry {
    fn from_object(obj: &DentryObject, name: &str) -> Self {
        Self {
            name: name.to_owned(),
            kind: obj.kind,
            attr: FlagSet::new(obj.attr).unwrap(),
            created_at: obj.created_at,
            modified_at: obj.modified_at,
            children: None,
        }
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn kind(&self) -> &DentryType {
        &self.kind
    }

    pub fn attributes(&self) -> &FlagSet<DentryAttribute> {
        &self.attr
    }

    pub fn children(&self) -> Option<&[Dentry]> {
        self.children.as_ref().map(|f| f as &[Dentry])
    }
}

pub fn check(path: &str) -> Result<Dentry, FileSystemError> {
    unsafe {
        match sys_access(path) {
            Ok(size) => {
                let buffer = vec![0u8; size];
                match sys_inspect(path, &buffer) {
                    Ok(count) => read_dentry_from_object_bytes(&buffer, count),
                    Err(err) => Err(FileSystemError::from(err)),
                }
            }
            Err(err) => Err(FileSystemError::from(err)),
        }
    }
}

unsafe fn read_dentry_from_object_bytes(
    bytes: &[u8],
    count: usize,
) -> Result<Dentry, FileSystemError> {
    if count > 0 {
        let size = size_of::<DentryObject>();
        let first = &*(bytes.as_ptr() as *const DentryObject);
        let mut dentry = Dentry::from_object(
            first,
            core::str::from_utf8_unchecked(&bytes[size..(size + first.name_length)]),
        );
        match first.kind {
            DentryType::Directory => {
                if count > 1 {
                    let mut children = Vec::<Dentry>::with_capacity(count - 1);
                    let mut pointer = size + (first.name_length + 8 - 1) & !(8 - 1);
                    for _ in 0..(count - 1) {
                        let this = &*(bytes.as_ptr().add(pointer) as *const DentryObject);
                        let child = Dentry::from_object(
                            this,
                            core::str::from_utf8_unchecked(
                                &bytes[(pointer + size)..(pointer + size + this.name_length)],
                            ),
                        );
                        pointer += size + ((this.name_length + 8 - 1) & !(8 - 1));
                        children.push(child);
                    }
                    dentry.children = Some(children);
                } else {
                    dentry.children = Some(Vec::new());
                }
            }
            DentryType::MountPoint => {
                if count > 1 {
                    let mounted = read_dentry_from_object_bytes(
                        &bytes[(size + ((first.name_length + 8 - 1) & !(8 - 1)))..],
                        count - 1,
                    )?;
                    dentry.children = Some(vec![mounted]);
                } else {
                    panic!("no mounted info")
                }
            }
            _ => {
                // don't care
            }
        }
        Ok(dentry)
    } else {
        Err(FileSystemError::NotAvailable)
    }
}
