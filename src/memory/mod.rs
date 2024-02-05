pub mod allocator;

use core::ops::Add;

#[repr(C)]
#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub struct Address(pub usize);

impl Add<usize> for Address
{
    type Output = Address;

    #[inline(always)]
    fn add(self, rhs: usize) -> Self::Output
    {
        Self(self.0 + rhs)
    }
}

#[derive(Clone, Copy, Debug)]
pub enum RegionType
{
    Unknown = 0,
    Available = 1,
    Reserved = 2,
    ACPI = 3,
    NVS = 4,
    BadRAM = 5,
}
use RegionType::*;

#[derive(Clone, Copy, Debug)]
pub struct PhysicalRegion
{
        start: usize,
        size: usize,
        rtype: RegionType,
}

impl PhysicalRegion
{
    pub fn new(start: usize, size: usize, rtype: usize) -> Self
    {
        PhysicalRegion{
            start,
            size,
            rtype: match rtype {
                1 => Available,
                2 => Reserved,
                3 => ACPI,
                4 => NVS,
                5 => BadRAM,
                _ => Unknown
            }
        }
    }
}



// Static structure to hold information about memrory banks
pub  static mut PHYS_MEM : [PhysicalRegion; 10] = [PhysicalRegion {start : 0, size: 0, rtype: Unknown}; 10];

pub const PAGE_SIZE : usize = crate::arch::PAGE_SIZE;



// TODO port the architture independent code here
