use core::fmt::{Display, Formatter, Result};

use alloc::string::String;
use erhino_shared::mem::Address;

pub struct BoardInfo {
    pub name: String,
    pub base_frequency: Address,
    pub mswi_address: Address,
    pub mtimer_address: Address,
    pub mtime_address: Address
}

impl Display for BoardInfo {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        write!(f, "{} {{ ", self.name)?;
        write!(
            f,
            "frequency={:#x}, mswi={:#x}, mtimer={:#x}",
            self.base_frequency, self.mswi_address, self.mtimer_address
        )?;
        write!(f, " }}")
    }
}
