mod bump;
mod listalloc;
pub mod mapper;

use crate::klib::lock::RwLock;
use listalloc::ListAllocator;

enum AllocError {
    ENOMEM,
}
 
#[global_allocator]
static mut ALLOCATOR : RwLock<Option<ListAllocator>> = RwLock::new(None); 

pub trait KernelAllocator 
{
    /// Initialize the allocator on a virtual region starting at @memstart, with @size, and a
    /// @base_pages number to allocate to start with
    fn init(&mut self, memstart: usize, heapsize: usize);
}

#[inline(always)]
pub fn init(memstart: usize, size: usize)
{
    unsafe { 
        ALLOCATOR = RwLock::new(Some(ListAllocator::default()));
        let mut guard = ALLOCATOR.write().unwrap();
        guard.as_mut().unwrap().init(memstart, size);
    };
}
