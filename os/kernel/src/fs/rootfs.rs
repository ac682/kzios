use alloc::{borrow::ToOwned, string::String, vec::Vec};
use erhino_shared::{
    fal::{Dentry, DentryAttribute, DentryKind, FileSystem, FileAbstractLayerError, Mid},
    mem::Address,
    path::{Component, Path, PathIterator},
    proc::Pid,
    sync::InteriorLock,
};
use flagset::FlagSet;

use crate::sync::{mutex::SpinLock, up::UpSafeCell};

use super::procfs::Procfs;

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
            kind: LocalDentryKind::Directory(UpSafeCell::new(Vec::new()), SpinLock::new()),
        }
    }

    pub fn new_mountpoint(name: &str, reference: Mid) -> Self {
        Self {
            name: name.to_owned(),
            kind: LocalDentryKind::MountPoint(reference),
            attr: DentryAttribute::None.into(),
        }
    }

    pub fn real(&self) -> &LocalDentryKind {
        &self.kind
    }
}

impl Dentry for LocalDentry {
    fn name(&self) -> &str {
        &self.name
    }

    fn attributes(&self) -> &FlagSet<DentryAttribute> {
        &self.attr
    }

    fn kind(&self) -> &DentryKind {
        todo!()
    }
}

pub enum LocalDentryKind {
    Directory(UpSafeCell<Vec<Node>>, SpinLock),
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

    pub fn mount(&self, path: &Path, mountpoint: Mid) -> Result<(), FileAbstractLayerError> {
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
            Err(FileAbstractLayerError::InvalidPath)
        }
    }

    fn get_parent(&self, path: &Path) -> Result<&Node, FileAbstractLayerError> {
        if path.is_absolute() {
            if let Ok(qualified) = path.qualify() {
                if let Some(parent) = qualified.parent() {
                    if let Ok(p) = Path::from(parent) {
                        let mut iter = p.iter();
                        iter.next();
                        let node = Self::find_node(&self.root, iter)?;
                        if let LocalDentryKind::Directory(_, _) = node.real() {
                            Ok(node)
                        } else {
                            Err(FileAbstractLayerError::Mistyped)
                        }
                    } else {
                        Err(FileAbstractLayerError::InvalidPath)
                    }
                } else {
                    Err(FileAbstractLayerError::InvalidPath)
                }
            } else {
                Err(FileAbstractLayerError::InvalidPath)
            }
        } else {
            Err(FileAbstractLayerError::InvalidPath)
        }
    }

    fn create_node(&self, parent: &Path, dentry: LocalDentry) -> Result<(), FileAbstractLayerError> {
        if parent.is_absolute() {
            if let Ok(qualified) = parent.qualify() {
                let mut iter = qualified.iter();
                iter.next();
                match Self::find_node(&self.root, parent.iter()) {
                    Ok(directory) => {
                        if let LocalDentryKind::Directory(subs, lock) = directory.real() {
                            // subs 可以用 hashmap 或者以 filename 为 key 的 btree 优化一下
                            lock.lock();
                            let mut found = false;
                            for i in subs.iter() {
                                if i.name() == dentry.name() {
                                    found = true;
                                    break;
                                }
                            }
                            if found {
                                subs.get_mut().push(UpSafeCell::new(dentry));
                                lock.unlock();
                                Ok(())
                            } else {
                                lock.unlock();
                                Err(FileAbstractLayerError::Conflict)
                            }
                        } else {
                            Err(FileAbstractLayerError::Mistyped)
                        }
                    }
                    Err(e) => Err(e),
                }
            } else {
                Err(FileAbstractLayerError::InvalidPath)
            }
        } else {
            Err(FileAbstractLayerError::InvalidPath)
        }
    }

    fn find_node<'a>(
        container: &'a Node,
        mut path: PathIterator,
    ) -> Result<&'a Node, FileAbstractLayerError> {
        if let Some(next) = path.next() {
            match next {
                Component::Normal(name) => match container.real() {
                    LocalDentryKind::Directory(subs, lock) => {
                        lock.lock();
                        for s in subs.iter() {
                            if s.name() == name {
                                lock.unlock();
                                return Self::find_node(s, path);
                            }
                        }
                        lock.unlock();
                        Err(FileAbstractLayerError::NotFound)
                    }
                    LocalDentryKind::Link(target) => Err(FileAbstractLayerError::ForeignLink(
                        path.collect_remaining(),
                        target.to_owned(),
                    )),
                    LocalDentryKind::File(_) => Err(FileAbstractLayerError::Mistyped),
                    LocalDentryKind::MountPoint(mountpoint) => {
                        Err(FileAbstractLayerError::ForeignMountPoint(
                            path.collect_remaining(),
                            mountpoint.to_owned(),
                        ))
                    }
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

    fn lookup(&self, path: Path) -> Result<&dyn Dentry, FileAbstractLayerError> {
        todo!()
    }
}
