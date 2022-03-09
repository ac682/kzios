use core::{
    ops::{Add, AddAssign, SubAssign},
    slice::from_raw_parts_mut,
};

use crate::config::{PAGE_SIZE, PAGE_SIZE_BITS};

/// 39 bits(u64)
/// 38-30   29-21   20-12   11-0
/// VPN2(9) VPN1(9) VPN0(9) Offset(12)
#[derive(Debug, Clone, Copy, PartialEq, PartialOrd)]
pub struct VirtualAddress(u64);

impl VirtualAddress {
    pub fn get_page_number(&self) -> (u16, u16, u16) {
        return (
            ((self.0 >> 12) & 0x1ff) as u16,
            ((self.0 >> 21) & 0x1ff) as u16,
            ((self.0 >> 30) & 0x1ff) as u16,
        );
    }

    pub fn get_offset(&self) -> u16 {
        return (self.0 & 0xfff) as u16;
    }
}

impl From<VirtualAddress> for u64{
    fn from(v: VirtualAddress) -> Self {
        v.0
    }
}

impl From<u64> for VirtualAddress{
    fn from(v: u64) -> Self{
        Self(v)
    }
}

/// 56 bits(u64)
/// 55-30       29-21   20-12   11-0
/// PPN2(26)    PPN1(9) PPN0(9) Offset(12)
#[derive(Debug, Clone, Copy, PartialEq, PartialOrd)]
pub struct PhysicalAddress(u64);

impl PhysicalAddress {

    pub fn get_mut<T>(&self) -> &'static mut T {
        unsafe { (self.0 as *mut T).as_mut().unwrap() }
    }

    pub fn get_mut_offset<T>(&self, offset: u64) -> &'static mut T {
        unsafe { ((self.0 + offset) as *mut T).as_mut().unwrap() }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, PartialOrd)]
pub struct PhysicalPageNumber(u64);

impl PhysicalPageNumber {
    pub fn from_address(v: u64) -> Self {
        PhysicalAddress::from(v).into()
    }

    pub fn get_frame(&self) -> &'static mut [u8] {
        unsafe { from_raw_parts_mut(PhysicalAddress::from(*self).0 as *mut u8, PAGE_SIZE) }
    }

    pub fn get_mut<T>(&self) -> &'static mut T {
        let address: PhysicalAddress = (*self).into();
        address.get_mut()
    }

    pub fn clear_frame(&mut self){
        let bytes = self.get_frame();
        for byte in bytes {
            *byte = 0;
        }
    }
}

impl From<PhysicalAddress> for PhysicalPageNumber {
    fn from(v: PhysicalAddress) -> Self {
        PhysicalPageNumber(v.0 >> PAGE_SIZE_BITS)
    }
}

impl From<PhysicalPageNumber> for PhysicalAddress {
    fn from(v: PhysicalPageNumber) -> Self {
        Self(v.0 << PAGE_SIZE_BITS)
    }
}

impl From<u64> for PhysicalAddress {
    fn from(v: u64) -> Self {
        PhysicalAddress(v)
    }
}

impl From<PhysicalAddress> for u64 {
    fn from(v: PhysicalAddress) -> Self {
        v.0
    }
}

impl From<PhysicalPageNumber> for u64 {
    fn from(v: PhysicalPageNumber) -> Self {
        v.0
    }
}

impl From<u64> for PhysicalPageNumber {
    fn from(v: u64) -> Self {
        PhysicalPageNumber(v)
    }
}

impl Add<u64> for PhysicalPageNumber {
    type Output = PhysicalPageNumber;

    fn add(self, rhs: u64) -> Self::Output {
        PhysicalPageNumber(self.0 + rhs)
    }
}

impl AddAssign<u64> for PhysicalPageNumber {
    fn add_assign(&mut self, rhs: u64) {
        self.0 = self.0 + rhs
    }
}

impl SubAssign<u64> for PhysicalPageNumber {
    fn sub_assign(&mut self, rhs: u64) {
        self.0 = self.0 - rhs
    }
}
