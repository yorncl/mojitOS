// use super::super::Address;

use crate::klog;

use crate::memory::BUMP_ALLOCATOR;


// Bump allocator, used to bootstrap the other allocators
pub struct Bump
{
    pub start: usize, // TOOD shoudl I use crate::memory::Address type ?
    pub size: usize
}

pub struct RawBox<T>
{
        pub data: *mut T,
}

impl<T> RawBox<T>
{
    pub fn new(s: T) -> Self
    {
        unsafe {
            let pointer = BUMP_ALLOCATOR.allocate(core::mem::size_of::<T>());
            // TODO memcpy
            RawBox { data : pointer as *mut T }
        }
    }

    pub fn from_ptr(ptr: *const T) -> Self
    {
            klog!("From ptr : {:p}", ptr);
            RawBox { data : ptr as *mut T }
    }

    #[inline(always)]
    pub fn to_ptr(&self) -> *const T
    {
        self.data
    }
}

impl<T> Default for RawBox<T>
{
    fn default() -> Self
    {
            RawBox { data : 0 as *mut T }
    }
}

impl<T> Deref for RawBox<T>
{
    type Target = T; // TODO why does it need that if derefmut does not ?

    #[inline(always)]
    fn deref(&self) -> &Self::Target
    {
        unsafe {&(*self.data)}
    }
}

impl<T> DerefMut for RawBox<T>
{
    #[inline(always)]
    fn deref_mut(&mut self) -> &mut Self::Target
    {
        unsafe {&mut(*self.data)}
    }
}

use core::{fmt, ops::{Deref, DerefMut}};

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

