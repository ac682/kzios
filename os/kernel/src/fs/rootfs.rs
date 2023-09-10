use alloc::{borrow::ToOwned, string::String, vec::Vec};
use erhino_shared::{
    fal::{Dentry, DentryAttribute, DentryMeta, FileSystem, FilesystemAbstractLayerError, Mid},
    mem::Address,
    path::{Component, Path, PathIterator},
    sync::spin::QueueLock,
};

use flagset::FlagSet;
use lock_api::RawMutex;

use crate::sync::up::UpSafeCell;

type Node = UpSafeCell<LocalDentry>;

// 除了 directory 其他都是只读的，文件也是，因为只是元数据，只有目录是结构数据，需要结构锁
pub struct LocalDentry {
    name: String,
    kind: LocalDentryKind,
    attr: FlagSet<DentryAttribute>,
}

impl LocalDentry {
    pub fn new_directory<A: Into<FlagSet<DentryAttribute>>>(name: &str, attr: A) -> Self {
        Self {
            name: name.to_owned(),
            attr: attr.into(),
            kind: LocalDentryKind::Directory(UpSafeCell::new(Vec::new()), QueueLock::new()),
        }
    }

    pub fn new_mountpoint(name: &str, reference: Mid) -> Self {
        Self {
            name: name.to_owned(),
            kind: LocalDentryKind::MountPoint(reference),
            attr: DentryAttribute::None.into(),
        }
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn kind(&self) -> &LocalDentryKind {
        &self.kind
    }

    pub fn meta(&self, collect: bool) -> Dentry {
        Dentry::new(
            self.name.clone(),
            self.attr.clone(),
            match self.kind() {
                LocalDentryKind::Directory(subs, lock) => {
                    if collect {
                        lock.lock();
                        let meta = DentryMeta::Directory(
                            subs.iter().map(|d| d.meta(false)).collect::<Vec<Dentry>>(),
                        );
                        unsafe { lock.unlock() };
                        meta
                    } else {
                        DentryMeta::Directory(Vec::new())
                    }
                }
                LocalDentryKind::MountPoint(mid) => DentryMeta::MountPoint(*mid),
                _ => todo!(),
            },
        )
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
    Property(),
}

pub struct Rootfs {
    root: Node,
}

impl Rootfs {
    pub fn new() -> Self {
        Self {
            root: UpSafeCell::new(LocalDentry::new_directory(
                "",
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
                LocalDentry::new_mountpoint(path.filename(), mountpoint),
            )
        } else {
            Err(FilesystemAbstractLayerError::InvalidPath)
        }
    }

    fn get_parent(&self, path: &Path) -> Result<&Node, FilesystemAbstractLayerError> {
        if path.is_absolute() {
            if let Ok(qualified) = path.qualify() {
                if let Some(parent) = qualified.parent() {
                    if let Ok(p) = Path::from(parent) {
                        let mut iter = p.iter();
                        iter.next();
                        let node = Self::find_node_internal(&self.root, iter)?;
                        if let LocalDentryKind::Directory(_, _) = node.kind() {
                            Ok(node)
                        } else {
                            Err(FilesystemAbstractLayerError::Mistyped)
                        }
                    } else {
                        Err(FilesystemAbstractLayerError::InvalidPath)
                    }
                } else {
                    Err(FilesystemAbstractLayerError::InvalidPath)
                }
            } else {
                Err(FilesystemAbstractLayerError::InvalidPath)
            }
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
            if let Ok(qualified) = path.qualify() {
                let mut iter = qualified.iter();
                iter.next();
                Self::find_node_internal(&self.root, iter)
            } else {
                Err(FilesystemAbstractLayerError::InvalidPath)
            }
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
    // 等日后写出 ramfs 的解决方案了再让 rootfs 变成 ramfs 并导入 initfs 的内容
    fn is_property_supported(&self) -> bool {
        false
    }

    fn is_stream_supported(&self) -> bool {
        false
    }

    fn lookup(&self, path: Path) -> Result<Dentry, FilesystemAbstractLayerError> {
        self.find_node(&path).map(|d| d.meta(true))
    }
}
