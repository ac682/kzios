use erhino_shared::call::SystemCallError;

use crate::call::sys_access;

pub enum FileSystemError {
    Unknown,
    InvalidPath,
    NotFound,
    NotAccessible,
    // Filesystem mountpoint does not exist
    NotAvailable
}

pub fn access(path: &str) -> Result<usize, FileSystemError> {
    unsafe {
        sys_access(path).map_err(|e| match e {
            SystemCallError::IllegalArgument => FileSystemError::InvalidPath,
            SystemCallError::ObjectNotAccessible => FileSystemError::NotAccessible,
            SystemCallError::ObjectNotFound => FileSystemError::NotFound,
            _ => FileSystemError::Unknown,
        })
    }
}
