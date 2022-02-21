struct PhysicalAddress(usize);

impl From<usize> for PhysicalAddress
{
    fn from(value: usize) -> Self {
        PhysicalAddress(value)
    }
}