use core::alloc::GlobalAlloc;
use core::borrow::{BorrowMut, Borrow};
use super::{KernelAllocator, AllocError};
use crate::memory::{pmm, PAGE_SIZE};
use crate::memory::vmm::mapper;
use crate::x86::paging::ROUND_PAGE_UP;
use alloc::alloc::Layout;
use core::mem::size_of;
use crate::{klog, align_up, align_down, is_aligned, kprint};
use super::Lock;
use core::fmt;

pub struct ListAllocator
{
    head: BlockInfo,
    // Virtual address of the heap's start
    memstart: usize,
    /// Current size of heap
    heapsize: usize,
    /// Max size of heap
    heapmax: usize,
}

impl KernelAllocator for ListAllocator
{
    /// Init list allocator parameters
    fn init(&mut self, memstart: usize, heapmax: usize) {
        self.memstart = memstart; 
        self.heapmax = heapmax;
    }
}

/// Metadata about free block
/// Attached to every free block
#[repr(C)]
struct BlockInfo {
    size: usize,
    next: Option<&'static mut BlockInfo>
}

impl fmt::Debug for BlockInfo {

    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result
    {
        f.debug_struct("PhysicalRegion")
            .field("size", &format_args!("{}KB ({}B)", self.size/1024, self.size))
            .field("next", &format_args!("{:p}", self as *const BlockInfo))
            .finish()
    }
}
 
macro_rules! binfo_size{
    () => {
       core::mem::size_of::<BlockInfo>()
    };
}

impl BlockInfo {
    #[inline(always)]
    pub fn data(&self) -> *mut u8
    {
        (self as *const BlockInfo as usize + size_of::<Self>()) as *mut u8
    }
}

impl ListAllocator
{
    pub fn default() -> Self {
        ListAllocator { 
            head: BlockInfo {size: 0, next: None},
            memstart: 0,
            heapmax: 0,
            heapsize: 0,
        }
    }

    // Release block and insert it in free list, coalesce with neighbours, unmap if last 
    fn free_block(&mut self, block: &BlockInfo)
    {
        klog!("Freeing {:?}", self);
        loop {}
        // klog!("Freeing block");
        // let address = block as *const BlockInfo as usize;

        // let mut current = &self.head;
        // while let Some(n) = current.next {
        //     let address = n.data() as usize;
        // }
    }

    /// Tries to find an existing free block of sufficient enough size
    /// This function will expand the heap if it doesn't find one
    fn alloc_block(&mut self, new_size: usize) -> Result<*mut u8, AllocError>
    {
        klog!("fn alloc_block {}B", new_size);
        let mut current = self.head.borrow_mut();
        // Loop through existing free blocks to find match
        while let Some(ref mut b) = current.next { // TODO how is b type legal
            // If exact match
            let fbsize = b.size;
            if fbsize == new_size {
                // Removing node from list
                let next = b.next.take();
                let ret = current.next.take().unwrap();
                current.next = next;
                // b.next = None;
                return Ok(ret as *const BlockInfo as *mut u8);
            }
            // if the requested size is smaller, and the difference can contain a new blockinfo
            else if fbsize > new_size
                && is_aligned!(b.size - new_size, core::mem::align_of::<BlockInfo>()) {// TODO is this alignment check rational ?
                unsafe {
                    let left = BlockInfo { size: b.size - new_size, next: b.next.take()};
                    let left_ptr = *b as *mut BlockInfo;
                    // TODO might crash on large blocks
                    let _ = left_ptr.offset(b.size.try_into().unwrap());
                    left_ptr.write(left);
                    // Downsizing newly allocated block size
                    b.size = new_size;
                    // Getting its address
                    let ret = *b as *const BlockInfo as *mut u8;

                    // setting the current next to the leftovers node
                    current.next = left_ptr.as_mut();
                    // klog!("{:?}", b.next);
                    loop{}
                    return Ok(ret);
                }
            }
            current = current.next.as_mut().unwrap(); // TODO don't really understand this line
        }
        // At this point no match was found
        // If the current node has a size superior to 0, we can map the difference
        // if current.size > 0 {
        //     TODO
        // }

        // else we have to map enough page for the whole size
        // we need to align in case the leftover allocation needs another block
        let total = ROUND_PAGE_UP!(new_size + core::mem::align_of::<BlockInfo>());
        let start = self.memstart + self.heapsize;
        let range = match pmm::alloc_contiguous_pages(total/PAGE_SIZE) {
            Some(r) => r,
            None => { return Err(AllocError::ENOMEM) }
        };
        match mapper::map_range_kernel(range, start) {
            Ok(()) => {self.heapsize += total;}
            Err(()) => return Err(AllocError::ENOMEM)
        }

        let newblock = start as *const BlockInfo;
        unsafe {
            let nb = &mut*(newblock as *mut BlockInfo);
            // leftovers from allocating the pages
            // we need to allocate a new free block at the end of our list
            if total - new_size > 0 {
                let left = &mut *((newblock.offset(new_size.try_into().unwrap())) as *mut BlockInfo);
                *left = BlockInfo {
                    next : None,
                    size : total - new_size
                };
                // point last unused block next to leftovers
                current.next = Some(left);
            }
            nb.size = new_size;
            // The newly allocated block isn't in the free list, no neighbours
            nb.next = None;
        }
        Ok(newblock as *mut u8)
    }

    /// Decrease current heap size by a number of pages
    fn decrease_heap(&mut self, npages: usize)
    {
        mapper::unmap_range_kernel(self.memstart + self.heapsize - npages * PAGE_SIZE, npages).expect("Failed to decrease kernel heap");
        self.heapsize -= npages * PAGE_SIZE;
    }

    fn adjust_layout(layout: Layout) -> (usize, usize) {
        let layout = layout
            .align_to(core::mem::align_of::<BlockInfo>())
            .expect("adjusting alignment failed")
            .pad_to_align();
        let size = layout.size().max(core::mem::size_of::<BlockInfo>());
        (size, layout.align())
    }

    pub fn print_list(&self)
    {
        kprint!("fn print_list : head {:p}|", self as *const ListAllocator);
        let mut current = &self.head.next;
        let i = 0;
        loop {
            match current {
                Some(b) => {
                    kprint!("block {} {:?} {:p}| ", i, b.size, *b as *const BlockInfo);
                    current = &b.next; 
                }
                None => break
            }
        }
        kprint!("\n");
    }
}


unsafe impl GlobalAlloc for Lock<ListAllocator>
{
    unsafe fn alloc(&self, layout: Layout) -> *mut u8
    {
        let alloc = self.get();
        let (size, align) = ListAllocator::adjust_layout(layout);
        match alloc.alloc_block(size + size_of::<BlockInfo>()) { // TODO better alignment
            // management
            Ok(b) => {
                klog!("{:?}", alloc);
                alloc.print_list();
                return b as *const BlockInfo as *mut u8;
            }
            Err(_e) => return core::ptr::null_mut()
        }
    }
    unsafe fn dealloc(&self, ptr: *mut u8, layout: Layout)
    {
        let alloc = self.get();
        // TODO check aligntment 
        // TODO Add a mechanism to check if the pointer is valid ?
        let block: &BlockInfo =  &*{(ptr as usize - binfo_size!()) as *const BlockInfo};
        alloc.free_block(block);
    }
}


impl fmt::Debug for ListAllocator {

    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result
    {
        f.debug_struct("ListAllocator")
            .field("memstart", &format_args!("0x{:x}", self.memstart))
            .field("heapsize", &format_args!("{}KB ({}B)", self.heapsize/1024, self.heapsize))
            .field("heapmax", &self.heapmax)
            .finish()
    }
}
