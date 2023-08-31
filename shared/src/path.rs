use core::fmt::Display;

use alloc::{borrow::ToOwned, string::String};

/// The separator of path for nodes
pub const PATH_SEPARATOR: char = '/';
/// Refused characters for path
pub const INVALID_CHARACTERS: [char; 1] = ['\0'];

/// Parts of path
pub enum Component {
    /// /
    Root,
    /// .
    Current,
    /// ..
    Parent,
    /// Valid filename
    Normal,
}

/// Path parsing error
pub enum PathError {
    /// Containing zero byte
    InvalidCharacters,
}

/// Representing a file or directory
pub struct Path {
    inner: String,
}

impl Path {
    /// Construct a path from string
    pub fn new(s: &str) -> Result<Self, PathError> {
        if INVALID_CHARACTERS.iter().any(|c| s.contains(*c)) {
            Err(PathError::InvalidCharacters)
        } else {
            Ok(Self {
                inner: s.to_owned(),
            })
        }
    }

    /// Is string a valid file name(containing no /)
    pub fn is_filename(s: &str) -> bool {
        !s.contains(PATH_SEPARATOR)
    }

    /// Is path starting from root
    pub fn is_absolute(&self) -> bool {
        self.inner.starts_with(PATH_SEPARATOR)
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

    /// Into component iter
    pub fn components(&self) {
        // TODO: 待会写
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
}

impl Display for Path {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "{}", self.inner)
    }
}
