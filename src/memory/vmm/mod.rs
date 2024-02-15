mod bump;
mod listalloc;

pub mod mapper;

use listalloc::ListAllocator;


enum AllocError {
    ENOMEM,
}

#[global_allocator]
static mut ALLOCATOR : ListAllocator = ListAllocator::default_const();

pub trait KernelAllocator 
{
    /// Initialize the allocator on a virtual region starting at @memstart, with @size, and a
    /// @base_pages number to allocate to start with
    fn init(&mut self, memstart: usize, size: usize, base_pages: usize);
}


#[inline(always)]
pub fn init(memstart: usize, size: usize, base_pages: usize)
{
    unsafe {
        ALLOCATOR.init(memstart, size, base_pages);
    }
}
