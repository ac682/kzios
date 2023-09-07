use core::{fmt::Display, str::Split};

use alloc::{
    borrow::ToOwned,
    string::{String, ToString},
    vec::Vec,
};

/// The separator of path for nodes
pub const PATH_SEPARATOR: char = '/';
/// Refused characters for path
pub const INVALID_CHARACTERS: [char; 1] = ['\0'];

/// Parts of path
pub enum Component<'a> {
    /// /
    Root,
    /// .
    Current,
    /// ..
    Parent,
    /// Valid filename
    Normal(&'a str),
}

impl<'a> Component<'a> {
    pub fn as_str(&self) -> &str {
        match self {
            Component::Root => "",
            Component::Current => ".",
            Component::Parent => "..",
            Component::Normal(it) => it,
        }
    }
}

/// Path parsing error
#[derive(Debug)]
pub enum PathError {
    /// Containing zero byte
    InvalidCharacters,
    /// Making absolute path requires root node
    MissingRoot,
    /// Reaching the end of root node
    OverflowRoot,
}

/// Representing a file or directory
#[derive(Debug, Clone)]
pub struct Path {
    inner: String,
}

impl Path {
    /// Construct a path from string
    pub fn from(s: &str) -> Result<Self, PathError> {
        if Self::is_valid(s) {
            Ok(Self::from_string_unchecked(s.to_owned()))
        } else {
            Err(PathError::InvalidCharacters)
        }
    }

    /// Is string a valid file name(containing no /)
    pub fn is_filename(s: &str) -> bool {
        !s.contains(PATH_SEPARATOR)
    }

    /// Is string a valid path(containing no zero-byte)
    pub fn is_valid(s: &str) -> bool {
        !INVALID_CHARACTERS.iter().any(|c| s.contains(*c))
    }

    /// Is path starting from root
    pub fn is_absolute(&self) -> bool {
        self.inner.starts_with(PATH_SEPARATOR)
    }

    /// Construct an absolute path with no . or ..
    pub fn qualify(&self) -> Result<Path, PathError> {
        if self.is_absolute() {
            let mut buffer = Vec::<&str>::new();
            for c in self.iter() {
                match c {
                    Component::Current => {}
                    Component::Root => buffer.push(""),
                    Component::Parent => {
                        if buffer.len() > 1 {
                            buffer.pop();
                        } else {
                            return Err(PathError::OverflowRoot);
                        }
                    }
                    Component::Normal(normal) => buffer.push(normal),
                }
            }
            let path = buffer.join(&PATH_SEPARATOR.to_string());
            Path::from(&path)
        } else {
            Err(PathError::MissingRoot)
        }
    }

    /// Get its filename with no SEPARATOR char and parent
    pub fn filename(&self) -> &str {
        let non_separator_terminated = self.get_non_separated_terminated();
        if let Some(p) = Self::get_break_position(non_separator_terminated) {
            &non_separator_terminated[(p + 1)..]
        } else {
            &non_separator_terminated
        }
    }

    /// Get its parent full path regardless it's directory or file
    pub fn parent(&self) -> Option<&str> {
        let non_separator_terminated = self.get_non_separated_terminated();
        if let Some(p) = Self::get_break_position(non_separator_terminated) {
            Some(&non_separator_terminated[..p])
        } else {
            None
        }
    }

    /// Append sub path into it
    pub fn append(&mut self, s: &str) -> Result<&str, PathError> {
        if Self::is_valid(s) {
            if self.inner.ends_with(PATH_SEPARATOR) {
                if s.starts_with(PATH_SEPARATOR) {
                    self.inner.pop();
                }
                self.inner.push_str(s);
                Ok(&self.inner)
            } else {
                if !s.starts_with(PATH_SEPARATOR) {
                    self.inner.push(PATH_SEPARATOR);
                }
                self.inner.push_str(s);
                Ok(&self.inner)
            }
        } else {
            Err(PathError::InvalidCharacters)
        }
    }

    /// Get a iterator to iterate over path components
    pub fn iter(&self) -> PathIterator {
        PathIterator::from_path(self)
    }

    /// Get its str reference
    pub fn as_str(&self) -> &str {
        &self.inner
    }

    fn get_non_separated_terminated(&self) -> &str {
        let offset = if self.inner.ends_with(PATH_SEPARATOR) {
            // must be directory
            1
        } else {
            // file
            0
        };
        &self.inner[..(self.inner.len() - offset)]
    }

    fn get_break_position(non_separator_terminated: &str) -> Option<usize> {
        let mut position = 0usize;
        for c in non_separator_terminated.chars().rev() {
            if c == PATH_SEPARATOR {
                return Some(non_separator_terminated.len() - 1 - position);
            } else {
                position += c.len_utf8();
            }
        }
        None
    }

    fn from_string_unchecked(string: String) -> Self {
        Self { inner: string }
    }
}

impl Display for Path {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "{}", self.inner)
    }
}

/// Iterates over components
pub struct PathIterator<'a> {
    split: Split<'a, char>,
    has_root: bool,
}

impl<'a> PathIterator<'a> {
    fn from_path(path: &'a Path) -> Self {
        Self {
            split: path.get_non_separated_terminated().split(PATH_SEPARATOR),
            has_root: path.is_absolute(),
        }
    }

    pub fn collect_remaining(mut self) -> Path {
        let mut buffer = Vec::<String>::new();
        if let Some(first) = self.next() {
            match first {
                Component::Root => buffer.push("".to_owned()),
                _ => {
                    buffer.push("".to_owned());
                    buffer.push(first.as_str().to_owned())
                }
            }
        }
        while let Some(next) = self.next() {
            buffer.push(next.as_str().to_owned());
        }
        Path::from_string_unchecked(buffer.join("/"))
    }
}

impl<'a> Iterator for PathIterator<'a> {
    type Item = Component<'a>;

    fn next(&mut self) -> Option<Self::Item> {
        if let Some(next) = self.split.next() {
            Some(match next {
                "" => {
                    if self.has_root {
                        self.has_root = false;
                        Component::Root
                    } else {
                        Component::Normal("")
                    }
                }
                "." => Component::Current,
                ".." => Component::Parent,
                _ => Component::Normal(next),
            })
        } else {
            None
        }
    }
}
