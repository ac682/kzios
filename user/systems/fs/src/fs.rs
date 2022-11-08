use core::usize;

use alloc::boxed::Box;
use flagset::{flags, FlagSet};

pub enum FileSystemError {
    Unsupported,
    Unreachable,
    PathInvalid,
    NotFile,
    NotDirectory,
    NotMounted,
    FileNotFound,
    DirectoryNotFound,
    FileAlreadyExist,
    DirectoryAlreadyExist,
    DeviceUnavailable
}

pub enum FileType{
    Block,
    Character
}

flags! {
    pub enum FileAccessFlag: u8{
        Read,
        Write,
    }
}

// 文件系统不只是 block array 的管理，本质是系统上可操作的资源的树状集合
// 甚至处理器本身被挂载在文件系统上，作为系统的可管理资源暴露给进程/服务
pub trait FileSystem {
    fn make_directory(&mut self, path: &str) -> Result<(), FileSystemError>;
    fn remove_directory(&mut self, path: &str) -> Result<(), FileSystemError>;
}

pub trait Directory {
    fn is_empty(&self) -> bool;
    fn create_directory(&mut self, name: &str) -> Result<(), FileSystemError>;
    fn remove_directory(&mut self, name: &str) -> Result<(), FileSystemError>;
    fn mount(&mut self, fs: Box<dyn FileSystem>) -> Result<(), FileSystemError>;
}

pub trait File{
    fn open(&self) -> Result<Box<dyn Stream>, FileSystemError>;
}

// 可以是块设备也可以是字符设备，总之要有写和读操作（可以永远 Result::Err 但得有）
pub trait Stream {
    // 对于字符设备，遇到\0会停止接着读
    fn read(&self, buffer: &mut [u8]) -> Result<usize, FileSystemError>;
    fn write(&self, data: &[u8]) -> Result<usize, FileSystemError>;

    // 设置当前文件的游标，字符文件不支持（FileSystemError::Unsupported）
    fn seek(&self, position: usize) -> Result<usize, FileSystemError>;

    fn len(&self) -> usize;
}