use core::fmt::{Display, Formatter, Result};

use alloc::string::String;

pub struct BoardInfo {
    pub name: String,
    pub base_frequency: usize,
    pub mswi_address: usize,
    pub mtimer_address: usize,
    pub mtime_address: usize
}

impl Display for BoardInfo {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        write!(f, "{} {{ ", self.name)?;
        write!(
            f,
            "frequency={:#x}, mswi={:#x}, mtimer={:#x}, mtime={:#x}",
            self.base_frequency, self.mswi_address, self.mtimer_address, self.mtime_address
        )?;
        write!(f, " }}")
    }
}
