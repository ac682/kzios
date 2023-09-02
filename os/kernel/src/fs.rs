use erhino_shared::{
    fal::{Dentry, DentryAttribute, FileSystem},
    path::Path,
};
use spin::Once;

use crate::println;

use self::rootfs::{LocalDentry, Registry, Rootfs};

pub mod procfs;
pub mod rootfs;
pub mod sysfs;

static mut ROOT: Once<Rootfs> = Once::new();

pub fn init() {
    let mut rootfs = Rootfs::new();
    rootfs
        .make_dir(Path::from("/proc/srv/").unwrap(), DentryAttribute::Readable)
        .expect("msg");
    rootfs
        .make_dir(
            Path::from("/sys/devices/block").unwrap(),
            DentryAttribute::Writeable | DentryAttribute::Readable,
        )
        .expect("msg");
    rootfs
        .make_dir(
            Path::from("/sys/./../dev").unwrap(),
            DentryAttribute::Writeable | DentryAttribute::Readable,
        )
        .expect("msg");
    println!("{}", rootfs);
    unsafe {
        ROOT.call_once(|| rootfs);
    }
}

pub fn find(path: Path, local: fn(&LocalDentry), remote: fn(&Registry, Path)) -> bool{
    let root = unsafe { ROOT.get().unwrap() };
    todo!()
}
