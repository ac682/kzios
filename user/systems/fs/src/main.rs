#![no_std]

use alloc::{fmt::Write, string::String};
use rinlib::{
    fs::{self, check, components::Dentry},
    preclude::*,
    shared::{fal::DentryAttribute, path::Path},
};

fn main() {
    debug!("Hello, fs!");
    fs::create_directory(
        "/hello/",
        DentryAttribute::Readable | DentryAttribute::Executable | DentryAttribute::Writeable,
    )
    .unwrap();
    fs::create_property(
        "/hello/world",
        rinlib::shared::fal::PropertyKind::Integer,
        DentryAttribute::Readable | DentryAttribute::Writeable,
    )
    .unwrap();
    let mut buffer = String::from("All entries under root shown below\n");
    print_dir(Path::from("/").unwrap(), &mut buffer).unwrap();
    debug!("{}", buffer);
}

fn print_dir(path: Path, buffer: &mut String) -> core::fmt::Result {
    match check(path.as_str()) {
        Ok(dentry) => print_dentry(&dentry, &path, buffer),
        Err(err) => panic!("{}: {:?}", path.as_str(), err),
    }
}

fn print_dentry(dentry: &Dentry, path: &Path, buffer: &mut String) -> core::fmt::Result {
    match dentry {
        Dentry::Directory(directory) => {
            writeln!(buffer, "{}/", directory.name())?;
            let mut inner = String::new();
            for child in directory.children() {
                print_dir(path / child.name(), &mut inner)?;
            }
            for line in inner.split("\n") {
                if line != "" {
                    writeln!(buffer, "| {}", line)?;
                }
            }
            Ok(())
        }
        Dentry::Link(link) => {
            // TODO: link_read 来获取其 target
            writeln!(buffer, "@{} -> UNIMP", link.name())
        }
        Dentry::MountPoint(mountpoint) => {
            if let Some(mounted) = mountpoint.mounted() {
                write!(buffer, "[{}]", mountpoint.name())?;
                print_dentry(mounted, path, buffer)?;
                Ok(())
            } else {
                writeln!(buffer, "x[{}]", mountpoint.name())
            }
        }
        Dentry::Property(property) => {
            writeln!(
                buffer,
                "#{}: {:?}",
                property.name(),
                property.read().unwrap()
            )
        }
        Dentry::Stream(stream) => {
            writeln!(buffer, "{}: {}B", stream.name(), stream.size())
        }
        _ => todo!(),
    }
}
