use core::{cell::OnceCell, mem::size_of};

use alloc::{borrow::ToOwned, string::String, vec::Vec};
use erhino_shared::{
    fal::{
        Dentry, DentryAttribute, DentryMeta, DentryObject, DentryType, FileSystem,
        FilesystemAbstractLayerError, Mid,
    },
    path::Path,
    proc::Pid,
};
use flagset::FlagSet;

use crate::debug;

use self::{procfs::Procfs, rootfs::Rootfs};

pub mod procfs;
pub mod rootfs;
pub mod sysfs;

static mut ROOT: OnceCell<Rootfs> = OnceCell::new();
// 只会在 fs::init 中写，此后只读，就不上锁了
// Mid 为 (index + 1) << 32
static mut MOUNTPOINTS: Vec<LocalMountpoint> = Vec::new();

pub enum LocalMountpoint {
    Proc(Procfs),
}

pub fn init() {
    let rootfs = Rootfs::new();
    rootfs
        .mount(&Path::from("/proc").unwrap(), (0 + 1) << 32)
        .expect("mount /proc");
    unsafe {
        // slot id = 0, mid = 1 << 32
        MOUNTPOINTS.push(LocalMountpoint::Proc(Procfs::new()));
        ROOT.set(rootfs);
    }
}

pub fn make_dir<A: Into<FlagSet<DentryAttribute>>>(path: Path, recursive: bool, attr: A) {
    todo!()
}

pub fn mount_local(path: Path, slot: usize) -> Result<(), FilesystemAbstractLayerError> {
    unsafe { ROOT.get_mut().unwrap() }.mount(&path, ((slot + 1) << 32) as Mid)
}

pub fn mount_remote(path: Path, service: Pid) -> Result<(), FilesystemAbstractLayerError> {
    unsafe { ROOT.get_mut().unwrap() }.mount(&path, service as Mid)
}

fn redirect_with<T, O: Fn(&dyn FileSystem, Path) -> Result<T, FilesystemAbstractLayerError>>(
    op: O,
    fs: &impl FileSystem,
    path: Path,
) -> Result<T, FilesystemAbstractLayerError> {
    match op(fs, path) {
        Ok(dentry) => Ok(dentry),
        Err(err) => match err {
            FilesystemAbstractLayerError::ForeignMountPoint(rem, mid) => {
                debug!("redirected to {:#x} with {}", mid, rem);
                if let Some(fs) = get_local_fs(mid) {
                    match fs {
                        LocalMountpoint::Proc(proc) => redirect_with(op, proc, rem),
                    }
                } else {
                    Err(FilesystemAbstractLayerError::ForeignMountPoint(rem, mid))
                }
            }
            _ => Err(err),
        },
    }
}

pub fn lookup<'a>(path: Path) -> Result<Dentry, FilesystemAbstractLayerError> {
    redirect_with(
        |fs, p| fs.lookup(p),
        unsafe { ROOT.get_mut().unwrap() },
        path,
    )
}

pub fn get_local_index(mid: Mid) -> Option<usize> {
    if (mid >> 32) > 0 {
        Some(((mid >> 32) - 1) as usize)
    } else {
        None
    }
}

pub fn measure(dentry: &Dentry) -> usize {
    let meta = dentry.meta();
    let mut size = size_of::<DentryObject>() + ((dentry.name().len() + 8 - 1) & !(8 - 1));
    match &meta {
        DentryMeta::Directory(subs) => {
            for sub in subs {
                // 名字要按 8 byte 对齐
                size += size_of::<DentryObject>() + ((sub.name().len() + 8 - 1) & !(8 - 1));
            }
            size
        }
        DentryMeta::MountPoint(mid) => {
            if let Some(local) = get_local_fs(*mid) {
                if let Ok(mounted) = redirect_with(
                    |fs, p| fs.lookup(p),
                    match local {
                        LocalMountpoint::Proc(procfs) => procfs,
                    },
                    Path::from("/").unwrap(),
                ) {
                    size + measure(&mounted)
                } else {
                    todo!("no root");
                }
            } else {
                todo!("foreign")
            }
        }
        _ => size,
    }
}

pub fn make_objects<'a>(dentry: &Dentry, buffer: &'a mut Vec<(DentryObject, String)>) {
    let meta = dentry.meta();
    buffer.push((
        DentryObject::new(
            DentryType::from(meta),
            dentry.attributes(),
            dentry.created_at(),
            dentry.modified_at(),
            dentry.size(),
            false,
            dentry.name().len(),
        ),
        dentry.name().to_owned(),
    ));
    match &meta {
        DentryMeta::Directory(subs) => {
            for sub in subs {
                let sub_meta = sub.meta();
                buffer.push((
                    DentryObject::new(
                        DentryType::from(sub_meta),
                        sub.attributes(),
                        0,
                        0,
                        0,
                        false,
                        sub.name().len(),
                    ),
                    sub.name().to_owned(),
                ));
            }
        }
        DentryMeta::MountPoint(mid) => {
            if let Some(local) = get_local_fs(*mid) {
                if let Ok(mounted) = redirect_with(
                    |fs, p| fs.lookup(p),
                    match local {
                        LocalMountpoint::Proc(procfs) => procfs,
                    },
                    Path::from("/").unwrap(),
                ) {
                    make_objects(&mounted, buffer)
                } else {
                    todo!("no root");
                }
            } else {
                todo!("foreign")
            }
        }
        _ => {}
    }
}

pub fn get_local_fs(mid: Mid) -> Option<&'static LocalMountpoint> {
    if let Some(index) = get_local_index(mid) {
        let table = unsafe { &MOUNTPOINTS };
        if index < table.len() {
            Some(&table[index])
        } else {
            None
        }
    } else {
        None
    }
}

pub fn create<A: Into<FlagSet<DentryAttribute>>>(
    path: Path,
    kind: DentryType,
    attr: A,
) -> Result<(), FilesystemAbstractLayerError> {
    create_filesystem(unsafe { ROOT.get_mut().unwrap() }, path, kind, attr)
}

pub fn create_filesystem<A: Into<FlagSet<DentryAttribute>>>(
    fs: &impl FileSystem,
    path: Path,
    kind: DentryType,
    attr: A,
) -> Result<(), FilesystemAbstractLayerError> {
    todo!()
}

pub fn read(path: Path) -> Result<Vec<u8>, FilesystemAbstractLayerError> {
    redirect_with(|fs, p| fs.read(p), unsafe { ROOT.get_mut().unwrap() }, path)
}
