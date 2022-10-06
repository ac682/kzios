use core::fmt::{Display, Formatter, Result};

use alloc::string::String;

pub struct BoardInfo {
    pub name: String,
    pub mswi_address: usize,
    pub mtimer_address: usize,
}

impl Display for BoardInfo {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        write!(f, "{} {{ ", self.name)?;
        write!(f, "mswi={:#x}, mtimer={:#x}", self.mswi_address, self.mtimer_address)?;
        write!(f, " }}")
    }
}
