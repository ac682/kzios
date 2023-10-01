use core::mem::{size_of, size_of_val};

use alloc::{borrow::ToOwned, string::String, vec::Vec};
use erhino_shared::{
    fal::{
        Dentry, DentryAttribute, DentryMeta, DentryType, FileKind, FileSystem,
        FilesystemAbstractLayerError, Mid, PropertyKind,
    },
    mem::Address,
    path::{Component, Path, PathIterator},
    sync::spin::QueueLock,
    time::Timestamp,
};

use flagset::FlagSet;
use lock_api::RawMutex;

use crate::sync::up::UpSafeCell;

type Node = UpSafeCell<LocalDentry>;

// 除了 directory 其他都是只读的，文件也是，因为只是元数据，只有目录是结构数据，需要结构锁
pub struct LocalDentry {
    name: String,
    created: Timestamp,
    modified: Timestamp,
    kind: LocalDentryKind,
    attr: FlagSet<DentryAttribute>,
}

impl LocalDentry {
    pub fn new_directory<A: Into<FlagSet<DentryAttribute>>>(
        name: &str,
        created: Timestamp,
        modified: Timestamp,
        attr: A,
    ) -> Self {
        Self {
            name: name.to_owned(),
            created,
            modified,
            attr: attr.into(),
            kind: LocalDentryKind::Directory(UpSafeCell::new(Vec::new()), QueueLock::new()),
        }
    }

    pub fn new_mountpoint(
        name: &str,
        created: Timestamp,
        modified: Timestamp,
        reference: Mid,
    ) -> Self {
        Self {
            name: name.to_owned(),
            created,
            modified,
            kind: LocalDentryKind::MountPoint(reference),
            attr: DentryAttribute::None.into(),
        }
    }

    pub fn new_integer(
        name: &str,
        created: Timestamp,
        modified: Timestamp,
        attr: FlagSet<DentryAttribute>,
    ) -> Self {
        Self {
            name: name.to_owned(),
            created,
            modified,
            kind: LocalDentryKind::File(LocalFile::Property(LocalProperty::Integer(0))),
            attr,
        }
    }

    pub fn new_integers(
        name: &str,
        created: Timestamp,
        modified: Timestamp,
        attr: FlagSet<DentryAttribute>,
    ) -> Self {
        Self {
            name: name.to_owned(),
            created,
            modified,
            kind: LocalDentryKind::File(LocalFile::Property(LocalProperty::Integers(
                Vec::with_capacity(0),
            ))),
            attr,
        }
    }

    pub fn new_decimal(
        name: &str,
        created: Timestamp,
        modified: Timestamp,
        attr: FlagSet<DentryAttribute>,
    ) -> Self {
        Self {
            name: name.to_owned(),
            created,
            modified,
            kind: LocalDentryKind::File(LocalFile::Property(LocalProperty::Decimal(0f64))),
            attr,
        }
    }

    pub fn new_decimals(
        name: &str,
        created: Timestamp,
        modified: Timestamp,
        attr: FlagSet<DentryAttribute>,
    ) -> Self {
        Self {
            name: name.to_owned(),
            created,
            modified,
            kind: LocalDentryKind::File(LocalFile::Property(LocalProperty::Decimals(
                Vec::with_capacity(0),
            ))),
            attr,
        }
    }

    pub fn new_string(
        name: &str,
        created: Timestamp,
        modified: Timestamp,
        attr: FlagSet<DentryAttribute>,
    ) -> Self {
        Self {
            name: name.to_owned(),
            created,
            modified,
            kind: LocalDentryKind::File(LocalFile::Property(LocalProperty::String(
                String::with_capacity(0),
            ))),
            attr,
        }
    }

    pub fn new_blob(
        name: &str,
        created: Timestamp,
        modified: Timestamp,
        attr: FlagSet<DentryAttribute>,
    ) -> Self {
        Self {
            name: name.to_owned(),
            created,
            modified,
            kind: LocalDentryKind::File(LocalFile::Property(LocalProperty::Blob(
                Vec::with_capacity(0),
            ))),
            attr,
        }
    }

    pub fn new_memory_stream(
        name: &str,
        created: Timestamp,
        modified: Timestamp,
        attr: FlagSet<DentryAttribute>,
        address: Address,
        length: usize,
    ) -> Self {
        Self {
            name: name.to_owned(),
            created,
            modified,
            kind: LocalDentryKind::File(LocalFile::Stream(address, length)),
            attr,
        }
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn kind(&self) -> &LocalDentryKind {
        &self.kind
    }

    pub fn meta(&self, collect: bool) -> Dentry {
        match self.kind() {
            LocalDentryKind::Directory(subs, lock) => {
                if collect {
                    lock.lock();
                    let meta = DentryMeta::Directory(
                        subs.iter().map(|d| d.meta(false)).collect::<Vec<Dentry>>(),
                    );
                    unsafe { lock.unlock() };
                    Dentry::new(
                        self.name.clone(),
                        self.created,
                        self.modified,
                        0,
                        self.attr.clone(),
                        meta,
                    )
                } else {
                    Dentry::new(
                        self.name.clone(),
                        self.created,
                        self.modified,
                        0,
                        self.attr.clone(),
                        DentryMeta::Directory(Vec::new()),
                    )
                }
            }
            LocalDentryKind::MountPoint(mid) => Dentry::new(
                self.name.clone(),
                self.created,
                self.modified,
                0,
                self.attr.clone(),
                DentryMeta::MountPoint(*mid),
            ),
            LocalDentryKind::File(file) => match file {
                LocalFile::Stream(_, length) => Dentry::new(
                    self.name.to_owned(),
                    self.created,
                    self.modified,
                    *length,
                    self.attr.clone(),
                    DentryMeta::File(FileKind::Stream),
                ),
                LocalFile::Property(LocalProperty::Integer(_)) => Dentry::new(
                    self.name.to_owned(),
                    self.created,
                    self.modified,
                    size_of::<i64>(),
                    self.attr.clone(),
                    DentryMeta::File(FileKind::Property(PropertyKind::Integer)),
                ),
                LocalFile::Property(LocalProperty::Integers(it)) => Dentry::new(
                    self.name.to_owned(),
                    self.created,
                    self.modified,
                    size_of::<i64>() * it.len(),
                    self.attr.clone(),
                    DentryMeta::File(FileKind::Property(PropertyKind::Integers)),
                ),
                LocalFile::Property(LocalProperty::Decimal(_)) => Dentry::new(
                    self.name.to_owned(),
                    self.created,
                    self.modified,
                    size_of::<f64>(),
                    self.attr.clone(),
                    DentryMeta::File(FileKind::Property(PropertyKind::Decimal)),
                ),
                LocalFile::Property(LocalProperty::Decimals(it)) => Dentry::new(
                    self.name.to_owned(),
                    self.created,
                    self.modified,
                    size_of::<f64>() * it.len(),
                    self.attr.clone(),
                    DentryMeta::File(FileKind::Property(PropertyKind::Decimals)),
                ),
                LocalFile::Property(LocalProperty::String(it)) => Dentry::new(
                    self.name.to_owned(),
                    self.created,
                    self.modified,
                    it.len(),
                    self.attr.clone(),
                    DentryMeta::File(FileKind::Property(PropertyKind::String)),
                ),
                LocalFile::Property(LocalProperty::Blob(it)) => Dentry::new(
                    self.name.to_owned(),
                    self.created,
                    self.modified,
                    size_of::<u8>() * it.len(),
                    self.attr.clone(),
                    DentryMeta::File(FileKind::Property(PropertyKind::Blob)),
                ),
            },
            LocalDentryKind::Link(_) => Dentry::new(
                self.name.to_owned(),
                self.created,
                self.modified,
                0,
                self.attr.clone(),
                DentryMeta::Link,
            ),
        }
    }
}

pub enum LocalDentryKind {
    Directory(UpSafeCell<Vec<Node>>, QueueLock),
    Link(Path),
    File(LocalFile),
    MountPoint(Mid),
}

pub enum LocalFile {
    Stream(Address, usize),
    Property(LocalProperty),
}

pub enum LocalProperty {
    Integer(i64),
    Integers(Vec<i64>),
    Decimal(f64),
    Decimals(Vec<f64>),
    String(String),
    Blob(Vec<u8>),
}

pub struct Rootfs {
    root: Node,
}

impl Rootfs {
    pub fn new() -> Self {
        Self {
            root: UpSafeCell::new(LocalDentry::new_directory(
                "",
                0,
                0,
                DentryAttribute::Readable | DentryAttribute::Executable,
            )),
        }
    }

    pub fn mount(&self, path: &Path, mountpoint: Mid) -> Result<(), FilesystemAbstractLayerError> {
        // mount 的参数是 pid as service，但提供一种用户友好的方式，
        // 例如 mount_table 中 /mnt,/srv/fs/i_made_my_own_fat32，后者需要有对应进程自己 link 到自己的 /proc/{pid}
        // 然后由 /proc/{pid}/traits/fs/* 得到其支持的文件系统信息
        // Self::mount 不会检查 service traits，但用户接口中的 mount 会
        if let Some(parent) = path.parent() {
            self.create_node(
                &Path::from(parent).unwrap(),
                LocalDentry::new_mountpoint(path.filename(), 0, 0, mountpoint),
            )
        } else {
            Err(FilesystemAbstractLayerError::InvalidPath)
        }
    }

    pub fn create_stream(
        &self,
        path: &Path,
        address: Address,
        length: usize,
        attr: FlagSet<DentryAttribute>,
    ) -> Result<(), FilesystemAbstractLayerError> {
        if let Some(parent) = path.parent() {
            self.create_node(
                &Path::from(parent).unwrap(),
                LocalDentry::new_memory_stream(path.filename(), 0, 0, attr, address, length),
            )
        } else {
            Err(FilesystemAbstractLayerError::InvalidPath)
        }
    }

    fn create_node(
        &self,
        parent: &Path,
        dentry: LocalDentry,
    ) -> Result<(), FilesystemAbstractLayerError> {
        match self.find_node(parent) {
            Ok(directory) => {
                if let LocalDentryKind::Directory(subs, lock) = directory.kind() {
                    // subs 可以用 hashmap 或者以 filename 为 key 的 btree 优化一下
                    lock.lock();
                    let mut found = false;
                    for i in subs.iter() {
                        if i.name() == dentry.name() {
                            found = true;
                            break;
                        }
                    }
                    if !found {
                        subs.get_mut().push(UpSafeCell::new(dentry));
                        unsafe { lock.unlock() };
                        Ok(())
                    } else {
                        unsafe { lock.unlock() };
                        Err(FilesystemAbstractLayerError::Conflict)
                    }
                } else {
                    Err(FilesystemAbstractLayerError::Mistyped)
                }
            }
            Err(e) => Err(e),
        }
    }

    fn find_node(&self, path: &Path) -> Result<&Node, FilesystemAbstractLayerError> {
        if path.is_absolute() {
            let mut iter = path.iter();
            iter.next();
            Self::find_node_internal(&self.root, iter)
        } else {
            Err(FilesystemAbstractLayerError::InvalidPath)
        }
    }

    fn find_node_internal<'a>(
        container: &'a Node,
        mut path: PathIterator,
    ) -> Result<&'a Node, FilesystemAbstractLayerError> {
        if let Some(next) = path.next() {
            match next {
                Component::Normal(name) => match container.kind() {
                    LocalDentryKind::Directory(subs, lock) => {
                        lock.lock();
                        for s in subs.iter() {
                            if s.name() == name {
                                unsafe { lock.unlock() };
                                return Self::find_node_internal(s, path);
                            }
                        }
                        unsafe { lock.unlock() };
                        Err(FilesystemAbstractLayerError::NotFound)
                    }
                    LocalDentryKind::MountPoint(mountpoint) => {
                        let mut rem = path.collect_remaining();
                        if rem.prepend(name).is_ok() {
                            rem.make_root();
                            Err(FilesystemAbstractLayerError::ForeignMountPoint(
                                rem,
                                mountpoint.to_owned(),
                            ))
                        } else {
                            Err(FilesystemAbstractLayerError::InvalidPath)
                        }
                    }
                    _ => Err(FilesystemAbstractLayerError::Mistyped),
                },
                _ => unreachable!(),
            }
        } else {
            Ok(container)
        }
    }
}

impl FileSystem for Rootfs {
    fn is_property_supported(&self) -> bool {
        true
    }

    fn is_stream_supported(&self) -> bool {
        true
    }

    fn lookup(&self, path: Path) -> Result<Dentry, FilesystemAbstractLayerError> {
        self.find_node(&path).map(|d| d.meta(true))
    }

    fn create(
        &self,
        path: Path,
        kind: DentryType,
        attr: FlagSet<DentryAttribute>,
    ) -> Result<(), FilesystemAbstractLayerError> {
        if let Some(parent) = path.parent() {
            // 只支持内存属性，要添加内存流要用 Rootfs.create_stream
            if let Ok(dir) = Path::from(parent) {
                match kind {
                    DentryType::Directory => self.create_node(
                        &dir,
                        LocalDentry::new_directory(path.filename(), 0, 0, attr),
                    ),
                    DentryType::Integer => self
                        .create_node(&dir, LocalDentry::new_integer(path.filename(), 0, 0, attr)),
                    DentryType::Integers => self
                        .create_node(&dir, LocalDentry::new_integers(path.filename(), 0, 0, attr)),
                    DentryType::Decimal => self
                        .create_node(&dir, LocalDentry::new_decimal(path.filename(), 0, 0, attr)),
                    DentryType::Decimals => self
                        .create_node(&dir, LocalDentry::new_decimals(path.filename(), 0, 0, attr)),
                    DentryType::String => {
                        self.create_node(&dir, LocalDentry::new_string(path.filename(), 0, 0, attr))
                    }
                    DentryType::Blob => {
                        self.create_node(&dir, LocalDentry::new_blob(path.filename(), 0, 0, attr))
                    }
                    _ => Err(FilesystemAbstractLayerError::Unsupported),
                }
            } else {
                Err(FilesystemAbstractLayerError::InvalidPath)
            }
        } else {
            Err(FilesystemAbstractLayerError::InvalidPath)
        }
    }

    fn read(&self, path: Path) -> Result<Vec<u8>, FilesystemAbstractLayerError> {
        match self.find_node(&path) {
            Ok(dentry) => match &dentry.kind {
                LocalDentryKind::File(LocalFile::Property(prop)) => match prop {
                    LocalProperty::Integer(it) => Ok(i64::to_ne_bytes(*it).to_vec()),
                    LocalProperty::Integers(it) => {
                        Ok(it.iter().flat_map(|i| i64::to_ne_bytes(*i)).collect())
                    }
                    LocalProperty::Decimal(it) => Ok(f64::to_ne_bytes(*it).to_vec()),
                    LocalProperty::Decimals(it) => {
                        Ok(it.iter().flat_map(|i| f64::to_ne_bytes(*i)).collect())
                    }
                    LocalProperty::String(it) => Ok(it.bytes().collect()),
                    LocalProperty::Blob(it) => Ok(it.clone()),
                },
                LocalDentryKind::File(LocalFile::Stream(_addr, _length)) => {
                    Err(FilesystemAbstractLayerError::Unsupported)
                }
                _ => Err(FilesystemAbstractLayerError::Unsupported),
            },
            Err(err) => Err(err),
        }
    }
}
