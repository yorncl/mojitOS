use core::alloc::GlobalAlloc;
use super::{KernelAllocator, AllocError};
use crate::memory::pmm;
use alloc::alloc::Layout;

struct Node
{
        size: usize,
        next: Option<&'static Node>
}

pub struct ListAllocator
{
        head: Node,
        vmemstart: usize,
        size: usize,
}

impl KernelAllocator for ListAllocator
{
    fn init(&mut self, memstart: usize, size: usize, base_pages: usize) {
        self.vmemstart = memstart; 
        self.size = size;
        
        // allocate the contiguous base pages
        // 400KB for starters
        let range = pmm::alloc_contiguous_pages(100);

        let ptr = memstart as *mut Node;
        let n = Node {size, next: None};
        unsafe {
            ptr.write(n);
        }
        self.head.next = Some(unsafe {&*ptr});
    }
}

impl ListAllocator
{
    fn new() -> Self {
        Self::default()
    }

    pub const fn default_const() -> Self {
        ListAllocator { 
            head: Node {size: 0, next: None},
            vmemstart: 0,
            size: 0
        }
    }

    fn add_free_block(&mut self, memstart: usize, size: usize)
    {
        let mut current = &self.head;
        while let Some(n) = current.next {
            let address = n;

            current = n;
        }
    }

    fn alloc_block(&mut self, size: usize) -> Result<usize, AllocError>
    {
        // let current = &self.head;
        // while let n = Some(current.next) {
        //     let address = &n;
        // }
        Err(AllocError::ENOMEM)
    }

    fn free_block(&mut self, address: usize)
    {
    }
}

impl Default for ListAllocator
{
    fn default() -> Self {
        ListAllocator { 
            head: Node {size: 0, next: None},
            vmemstart: 0,
            size: 0
        }
    }
}


unsafe impl GlobalAlloc for ListAllocator
{
    // Required methods
    unsafe fn alloc(&self, layout: Layout) -> *mut u8
    {
        0 as *mut u8
    }
    unsafe fn dealloc(&self, ptr: *mut u8, layout: Layout)
    {}
}
