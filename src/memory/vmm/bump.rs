// use super::super::Address;

use crate::memory::BUMP_ALLOCATOR;


// Bump allocator, used to bootstrap the other allocators
pub struct Bump
{
    pub start: usize, // TOOD shoudl I use crate::memory::Address type ?
    pub size: usize
}

pub struct RawBox<T>
{
        data: *const T,
}

impl<T> RawBox<T>
{
    pub fn new(s: T) -> Self
    {
        unsafe {
            let pointer = BUMP_ALLOCATOR.allocate(core::mem::size_of::<T>());
            RawBox { data : pointer as *const T }
        }
    }
}

use core::fmt;

impl<T> fmt::Display for RawBox<T>
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "RawBox(at: {:p})", self.data)
    }
}


impl Bump
{
    fn new(start: usize, size: usize)
    {
        Bump {start, size};
    }

    pub fn allocate(&mut self, n: usize) -> usize
    {
        let offset = self.size;
        self.size += n;
        self.start + offset
        // TODO check for new pages
    }

}

