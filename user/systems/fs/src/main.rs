#![no_std]

use alloc::{fmt::Write, string::String};
use rinlib::{
    fs::{check, Dentry},
    preclude::*,
    shared::{fal::DentryType, path::Path},
};

fn main() {
    debug!("Hello, fs!");
    let mut buffer = String::from("All entries under root shown below\n");
    print_dir(Path::from("/").unwrap(), &mut buffer).unwrap();
    debug!("{}", buffer);
}

fn print_dir(path: Path, buffer: &mut String) -> core::fmt::Result {
    match check(path.as_str()) {
        Ok(dentry) => print_dentry(&dentry, &path, buffer),
        Err(err) => panic!("{:?}", err),
    }
}

fn print_dentry(dentry: &Dentry, path: &Path, buffer: &mut String) -> core::fmt::Result {
    match dentry.kind() {
        DentryType::MountPoint => {
            if let Some(children) = dentry.children() {
                if children.len() == 1 {
                    let mounted = &children[0];
                    write!(buffer, "!{}", dentry.name())?;
                    print_dentry(mounted, path, buffer)
                } else {
                    panic!("mountpoint no child");
                }
            } else {
                panic!("mountpoint no children");
            }
        }
        DentryType::Directory => {
            writeln!(buffer, "{}/", dentry.name())?;
            if let Some(children) = dentry.children() {
                let mut inner = String::new();
                for child in children {
                    print_dir(path / child.name(), &mut inner)?;
                }
                for line in inner.split("\n") {
                    if line != "" {
                        writeln!(buffer, "| {}", line)?;
                    }
                }
            }
            Ok(())
        }
        DentryType::Link => {
            writeln!(buffer, "{} -> ", dentry.name())
        }
        DentryType::Integer => {
            writeln!(buffer, "#{} = {:?}", dentry.name(), dentry.read(0).unwrap())
        }
        DentryType::Integers => {
            writeln!(buffer, "#{} = {:?}", dentry.name(), dentry.read(0).unwrap())
        }
        DentryType::Decimal => {
            writeln!(buffer, "#{} = {:?}", dentry.name(), dentry.read(0).unwrap())
        }
        DentryType::Decimals => {
            writeln!(buffer, "#{} = {:?}", dentry.name(), dentry.read(0).unwrap())
        }
        DentryType::Stream => {
            writeln!(buffer, "#{} = {:?}", dentry.name(), dentry.read(0).unwrap())
        }
        DentryType::Blob => {
            writeln!(buffer, "#{} = {:?}", dentry.name(), dentry.read(0).unwrap())
        }
        _ => todo!(),
    }
}
