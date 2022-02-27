use core::{ops::{Add, AddAssign, SubAssign}, slice::from_raw_parts_mut};

use crate::config::{PAGE_SIZE, PAGE_SIZE_BITS};



/// 39 bits(u64)
/// 38-30   29-21   20-12   11-0
/// VPN2(9) VPN1(9) VPN0(9) Offset(12)
struct VirtualAddress(u64);

impl VirtualAddress {
    pub fn page_number(&self) -> (u16, u16, u16) {
        return (
            ((self.0 >> 12) & 0x1ff) as u16,
            ((self.0 >> 21) & 0x1ff) as u16,
            ((self.0 >> 30) & 0x1ff) as u16,
        );
    }

    pub fn offset(&self) -> u16 {
        return (self.0 & 0xfff) as u16;
    }
}

/// 56 bits(u64)
/// 55-30       29-21   20-12   11-0
/// PPN2(26)    PPN1(9) PPN0(9) Offset(12)
pub struct PhysicalAddress(usize);

impl PhysicalAddress{
    pub fn get_mut<T>(&self) -> &'static mut T{
        unsafe {
            (self.0 as *mut T).as_mut().unwrap()
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, PartialOrd)]
pub struct PhysicalPageNumber(usize);

impl PhysicalPageNumber{
    pub fn from_address(v: usize) -> Self{
        PhysicalAddress::from(v).into()
    }

    pub fn get_frame(&self) -> &'static mut [u8] {
        unsafe { from_raw_parts_mut(PhysicalAddress::from(*self).0 as *mut u8, PAGE_SIZE) }
    }

    pub fn get_mut<T>(&self) -> &'static mut T{
        let address: PhysicalAddress = (*self).into();
        address.get_mut()
    }
}

impl From<PhysicalAddress> for PhysicalPageNumber {
    fn from(v: PhysicalAddress) -> Self {
        PhysicalPageNumber(v.0 >> PAGE_SIZE_BITS)
    }
}


impl From<PhysicalPageNumber> for PhysicalAddress{
    fn from(v: PhysicalPageNumber) -> Self {
        Self(v.0 << PAGE_SIZE_BITS)
    }
}

impl From<usize> for PhysicalAddress {
    fn from(v: usize) -> Self {
        PhysicalAddress(v)
    }
}

impl From<PhysicalAddress> for usize {
    fn from(v: PhysicalAddress) -> Self {
        v.0
    }
}

impl From<PhysicalPageNumber> for usize{
    fn from(v: PhysicalPageNumber) -> Self {
        v.0
    }
}

impl From<usize> for PhysicalPageNumber{
    fn from(v: usize) -> Self{
        PhysicalPageNumber(v)
    }
}

impl Add<usize> for PhysicalPageNumber{
    type Output = PhysicalPageNumber;

    fn add(self, rhs: usize) -> Self::Output {
        PhysicalPageNumber(self.0 + rhs)
    }
}

impl AddAssign<usize> for PhysicalPageNumber{
    fn add_assign(&mut self, rhs: usize) {
        self.0 = self.0 + rhs
    }
}

impl SubAssign<usize> for PhysicalPageNumber{
    fn sub_assign(&mut self, rhs: usize) {
        self.0 = self.0 - rhs
    }
}