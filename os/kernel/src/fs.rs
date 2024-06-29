use core::{cell::OnceCell, mem::size_of, ptr::addr_of_mut};

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
        .mount(&Path::from("/Processes").unwrap(), (0 + 1) << 32)
        .expect("mount /Processes");
    unsafe {
        // slot id = 0, mid = 1 << 32
        MOUNTPOINTS.push(LocalMountpoint::Proc(Procfs::new()));
        let _ = ROOT.set(rootfs);
    }
}

// NOTE: MountPoint 没有 Attributes，对于挂载点的操作会被转发到其挂载的目标文件系统的根目录上，包括 Execute 进入目录权限。
// 对于挂载点本身的操作都要用对应的挂载点系统调用进行，Access/Inspect 这种检查本身的则不会受权限影响。
// 对挂载点本身 Move 则会在挂载点所在的文件系统中移动挂载点，但是当对其 Delete 时则会对挂载的文件系统的根 Dentry 发送 Delete
pub fn mount_local(path: Path, slot: usize) -> Result<(), FilesystemAbstractLayerError> {
    unsafe { ROOT.get_mut().unwrap() }.mount(&path, ((slot + 1) << 32) as Mid)
}

pub fn mount_remote(path: Path, service: Pid) -> Result<(), FilesystemAbstractLayerError> {
    unsafe { ROOT.get_mut().unwrap() }.mount(&path, service as Mid)
}

pub fn create_memory_stream<A: Into<FlagSet<DentryAttribute>>>(
    path: Path,
    data: &[u8],
    attr: A,
) -> Result<(), FilesystemAbstractLayerError> {
    unsafe { ROOT.get_mut().unwrap() }.create_stream(
        &path,
        data.as_ptr() as usize,
        data.len(),
        attr.into(),
    )
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
        let table = unsafe { &mut *addr_of_mut!(MOUNTPOINTS) };
        if index < table.len() {
            Some(&table[index])
        } else {
            None
        }
    } else {
        None
    }
}

pub fn make_directory<A: Into<FlagSet<DentryAttribute>>>(
    path: Path,
    attr: A,
) -> Result<(), FilesystemAbstractLayerError> {
    make_directory_internal(path, attr.into())
}

fn make_directory_internal(
    path: Path,
    attr: FlagSet<DentryAttribute>,
) -> Result<(), FilesystemAbstractLayerError> {
    match lookup(path.clone()) {
        Ok(_) => Ok(()),
        Err(FilesystemAbstractLayerError::NotFound) => {
            if let Some(parent) = path.parent() {
                make_directory_internal(parent, attr.clone())?;
            }
            create(path, DentryType::Directory, attr)
        }
        Err(err) => Err(err),
    }
}

pub fn create<A: Into<FlagSet<DentryAttribute>>>(
    path: Path,
    kind: DentryType,
    attr: A,
) -> Result<(), FilesystemAbstractLayerError> {
    let flags = attr.into();
    redirect_with(
        |fs, p| fs.create(p, kind, flags),
        unsafe { ROOT.get_mut().unwrap() },
        path,
    )
}

pub fn write(path: Path, value: Vec<u8>) -> Result<(), FilesystemAbstractLayerError> {
    redirect_with(
        |fs, p| fs.write(p, &value),
        unsafe { ROOT.get_mut().unwrap() },
        path,
    )
}

pub fn read(path: Path, length: usize) -> Result<Vec<u8>, FilesystemAbstractLayerError> {
    redirect_with(
        |fs, p| fs.read(p, length),
        unsafe { ROOT.get_mut().unwrap() },
        path,
    )
}
