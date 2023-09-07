use alloc::vec::Vec;
use erhino_shared::{
    fal::{Dentry, DentryAttribute, FileSystem, FileAbstractLayerError, Mid},
    path::Path,
    proc::Pid,
};
use flagset::FlagSet;
use spin::Once;

use crate::println;

use self::{procfs::Procfs, rootfs::Rootfs};

pub mod procfs;
pub mod rootfs;
pub mod sysfs;

static mut ROOT: Once<Rootfs> = Once::new();
// 只会在 fs::init 中写，此后只读，就不上锁了
// Mid 为 (index + 1) << 32
static mut MOUNTPOINTS: Vec<LocalMountpoint> = Vec::new();

enum LocalMountpoint {
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
        ROOT.call_once(|| rootfs);
    }
}

pub fn make_dir<A: Into<FlagSet<DentryAttribute>>>(path: Path, recursive: bool, attr: A) {
    todo!()
}

pub fn mount_local(path: Path, slot: usize) -> Result<(), FileAbstractLayerError> {
    unsafe { ROOT.get_mut_unchecked() }.mount(&path, ((slot + 1) << 32) as Mid)
}

pub fn mount_remote(path: Path, service: Pid) -> Result<(), FileAbstractLayerError> {
    unsafe { ROOT.get_mut_unchecked() }.mount(&path, service as Mid)
}

pub fn access(path: Path) -> Result<usize, FileAbstractLayerError> {
    unsafe { ROOT.get_mut_unchecked() }
        .lookup(path)
        .map(|d| match d.kind() {
            _ => todo!(),
        })
}
