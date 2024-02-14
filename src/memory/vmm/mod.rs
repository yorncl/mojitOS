mod bump;
mod listalloc;

pub type Allocator = listalloc::ListAllocator;

enum AllocError {
    ENOMEM,
}

pub trait KernelAllocator 
{
    /// Initialize the allocator on a virtual region starting at @memstart, with @size, and a
    /// @base_pages number to allocate to start with
    fn init(&mut self, memstart: usize, size: usize, base_pages: usize);
}

// #[global_allocator]

