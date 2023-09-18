use core::mem::size_of;

use alloc::{borrow::ToOwned, string::String, vec, vec::Vec};
use erhino_shared::{
    call::SystemCallError,
    fal::{DentryAttribute, DentryObject, DentryType},
    path::Path,
    time::Timestamp,
};
use flagset::FlagSet;

use crate::{
    call::{sys_access, sys_inspect, sys_read},
    debug,
    ipc::tunnel::Runnel,
};

#[derive(Debug, Clone, Copy)]
pub enum FileSystemError {
    Unknown,
    InvalidPath,
    NotFound,
    UnsupportedOperation,
    NotAccessible,
    // Filesystem mountpoint does not exist
    NotAvailable,
    SystemError,
}

impl From<SystemCallError> for FileSystemError {
    fn from(value: SystemCallError) -> Self {
        match value {
            SystemCallError::IllegalArgument => FileSystemError::InvalidPath,
            SystemCallError::ObjectNotAccessible => FileSystemError::NotAccessible,
            SystemCallError::ObjectNotFound => FileSystemError::NotFound,
            SystemCallError::NotSupported => FileSystemError::UnsupportedOperation,
            SystemCallError::InternalError => FileSystemError::SystemError,
            _ => FileSystemError::Unknown,
        }
    }
}

pub enum DentryValue {
    Integer(i64),
    Integers(Vec<i64>),
    Decimal(f64),
    Decimals(Vec<f64>),
    String(String),
    Blob(Vec<u8>),
    Stream(Vec<u8>),
}

pub struct Dentry {
    identifier: String,
    name: String,
    kind: DentryType,
    attr: FlagSet<DentryAttribute>,
    size: usize,
    created_at: Timestamp,
    modified_at: Timestamp,
    children: Option<Vec<Dentry>>,
}

impl Dentry {
    fn from_object(obj: &DentryObject, name: &str, identifier: &str) -> Self {
        Self {
            identifier: identifier.to_owned(),
            name: name.to_owned(),
            kind: obj.kind,
            attr: FlagSet::new(obj.attr).unwrap(),
            size: obj.size as usize,
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

    pub fn size(&self) -> usize {
        self.size
    }

    pub fn children(&self) -> Option<&[Dentry]> {
        self.children.as_ref().map(|f| f as &[Dentry])
    }

    pub fn read(&self, count: usize) -> Result<DentryValue, FileSystemError> {
        match self.kind {
            DentryType::Integer
            | DentryType::Integers
            | DentryType::Decimal
            | DentryType::Decimals
            | DentryType::String
            | DentryType::Blob => unsafe {
                let mut buffer = vec![0u8; usize::min(self.size, count)];
                match sys_read(&self.identifier, &mut buffer) {
                    Ok(read) => {
                        buffer.truncate(read);
                        match self.kind {
                            DentryType::Integer => {
                                if read == 8 {
                                    Ok(DentryValue::Integer(i64::from_le_bytes([
                                        buffer[0], buffer[1], buffer[2], buffer[3], buffer[4],
                                        buffer[5], buffer[6], buffer[7],
                                    ])))
                                } else {
                                    Err(FileSystemError::SystemError)
                                }
                            }
                            DentryType::Integers => {
                                if read % 8 == 0 {
                                    let count = read / 8;
                                    let mut ints = Vec::<i64>::with_capacity(count);
                                    for i in 0..count {
                                        let int = i64::from_le_bytes([
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
                            DentryType::Decimal => {
                                if read == 8 {
                                    Ok(DentryValue::Decimal(f64::from_le_bytes([
                                        buffer[0], buffer[1], buffer[2], buffer[3], buffer[4],
                                        buffer[5], buffer[6], buffer[7],
                                    ])))
                                } else {
                                    Err(FileSystemError::SystemError)
                                }
                            }
                            DentryType::Decimals => {
                                if read % 8 == 0 {
                                    let count = read / 8;
                                    let mut decs = Vec::<f64>::with_capacity(count);
                                    for i in 0..count {
                                        let int = f64::from_le_bytes([
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
                            DentryType::String => {
                                buffer.truncate(count);
                                if let Ok(s) = String::from_utf8(buffer) {
                                    Ok(DentryValue::String(s))
                                } else {
                                    Err(FileSystemError::SystemError)
                                }
                            }
                            DentryType::Blob => {
                                buffer.truncate(count);
                                Ok(DentryValue::Blob(buffer))
                            }
                            _ => unreachable!(),
                        }
                    }
                    Err(err) => Err(FileSystemError::from(err)),
                }
            },
            DentryType::Stream => unsafe {
                let mut buffer = vec![0u8; usize::min(self.size, count)];
                match sys_read(&self.identifier, &mut buffer) {
                    Ok(read) => {
                        buffer.truncate(read);
                        Ok(DentryValue::Stream(buffer))
                    }
                    Err(err) => Err(FileSystemError::from(err)),
                }
            },
            _ => Err(FileSystemError::UnsupportedOperation),
        }
    }

    pub fn open(&self) -> Result<Runnel, FileSystemError> {
        todo!();
    }
}

unsafe fn read_dentry_from_object_bytes(
    bytes: &[u8],
    count: usize,
    request_path: &str,
) -> Result<Dentry, FileSystemError> {
    if count > 0 {
        let size = size_of::<DentryObject>();
        let first = &*(bytes.as_ptr() as *const DentryObject);
        let first_name =
            core::str::from_utf8_unchecked(&bytes[size..(size + first.name_length as usize)]);
        let mut dentry = Dentry::from_object(first, first_name, request_path);
        match first.kind {
            DentryType::Directory => {
                if count > 1 {
                    let mut children = Vec::<Dentry>::with_capacity(count - 1);
                    let mut pointer = size + (first.name_length as usize + 8 - 1) & !(8 - 1);
                    for _ in 0..(count - 1) {
                        let this = &*(bytes.as_ptr().add(pointer) as *const DentryObject);
                        let mut dir_path = Path::from(request_path).unwrap();
                        let name = core::str::from_utf8_unchecked(
                            &bytes[(pointer + size)..(pointer + size + this.name_length as usize)],
                        );
                        dir_path.append(name).unwrap();
                        let child = Dentry::from_object(this, name, dir_path.as_str());
                        pointer += size + ((this.name_length as usize + 8 - 1) & !(8 - 1));
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
                        &bytes[(size + ((first.name_length as usize + 8 - 1) & !(8 - 1)))..],
                        count - 1,
                        request_path,
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

pub fn check(path: &str) -> Result<Dentry, FileSystemError> {
    debug!("check {}", path);
    unsafe {
        match sys_access(path) {
            Ok(size) => {
                let buffer = vec![0u8; size];
                match sys_inspect(path, &buffer) {
                    Ok(count) => read_dentry_from_object_bytes(&buffer, count, path),
                    Err(err) => Err(FileSystemError::from(err)),
                }
            }
            Err(err) => Err(FileSystemError::from(err)),
        }
    }
}
