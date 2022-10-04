use core::fmt::{Display, Formatter, Result};

use alloc::string::String;

pub struct BoardInfo{
    pub name: String,
    pub mtimecmp_addr: usize
}

impl Display for BoardInfo{
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        write!(f, "Board: {}", self.name)
    }
}