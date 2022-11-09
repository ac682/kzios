use crate::fs::{Directory, FileSystem, Stream};
use alloc::{
    borrow::ToOwned,
    boxed::Box,
    collections::BTreeMap,
    string::{String, ToString},
    vec::{self, Vec},
};
use rinlib::dbg;
use unix_path::{Ancestors, Component, Components, Path};

use crate::fs::FileSystemError;

pub struct MemoryFs {
    mounted_at: String,
    root: Node,
}

enum Node {
    MountPoint(Box<dyn FileSystem>),
    Directory(BTreeMap<String, Node>),
    File(FileContent),
}

struct FileContent {
    data: Vec<u8>,
}

impl FileSystem for MemoryFs {
    fn make_directory(&mut self, path: &str) -> Result<(), FileSystemError> {
        self.get_entry_mut(path, |node| {
            match node {
                Node::MountPoint(_) => todo!(),
                Node::Directory(it) => it.insert(
                    Path::new(path).file_name().unwrap().to_str().unwrap().to_owned(),
                    Node::Directory(BTreeMap::new()),
                ),
                Node::File(_) => todo!(),
            };
        })
    }

    fn remove_directory(&mut self, path: &str) -> Result<(), FileSystemError> {
        todo!()
    }
}

impl MemoryFs {
    pub fn new(mount_point: &str) -> Self {
        Self {
            mounted_at: mount_point.to_string(),
            root: Node::Directory(BTreeMap::new()),
        }
    }

    pub fn print(&self) {
        dbg!("{}\n", self.mounted_at);
        Self::print_node(&self.root, 1);
    }

    fn print_node(node: &Node, level: usize) {
        for _ in 0..level {
            dbg!("  ");
        }
        match node {
            Node::MountPoint(_) => todo!(),
            Node::Directory(it) => {
                for (name, node) in it {
                    dbg!("{}", name);
                    Self::print_node(node, level + 1);
                }
            }
            Node::File(it) => dbg!("File {} bytes\n", it.data.len()),
        }
    }

    fn get_entry_mut(
        &mut self,
        path: &str,
        action: impl Fn(&mut Node) + Copy,
    ) -> Result<(), FileSystemError> {
        let buffer = Path::new(path);
        let relative = if buffer.is_absolute() {
            if let Ok(r) = buffer.strip_prefix(&self.mounted_at) {
                r
            } else {
                return Err(FileSystemError::PathInvalid);
            }
        } else {
            buffer
        };
        let mut components = relative.components();
        Self::get_entry_mut_internal(&mut components, action, &mut self.root)?;
        Ok(())
    }

    fn get_entry_mut_internal(
        relative: &mut Components,
        action: impl Fn(&mut Node) + Copy,
        node: &mut Node,
    ) -> Result<bool, FileSystemError> {
        match relative.next() {
            Some(Component::RootDir) => panic!("unreachable for relative is relative"),
            Some(Component::CurDir) => Self::get_entry_mut_internal(relative, action, node),
            Some(Component::ParentDir) => Ok(true),
            Some(Component::Normal(normal)) => match node {
                Node::File(_) => Err(FileSystemError::NotDirectory),
                Node::MountPoint(_) => todo!(),
                Node::Directory(it) => {
                    let name = normal.to_str().unwrap().to_owned();
                    if let Some(entry) = it.get_mut(&name) {
                        let rewind = Self::get_entry_mut_internal(relative, action, entry)?;
                        if rewind {
                            Ok(true)
                        } else {
                            action(entry);
                            Ok(false)
                        }
                    } else {
                        Err(FileSystemError::DirectoryNotFound)
                    }
                }
            },
            None => Ok(false),
        }
    }
}
