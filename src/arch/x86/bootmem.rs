
/// The boot memory allocator is supposed to manage early allocations, until the proper allocator kernel
/// manager is setup.
/// Among other things, it will help setup data for the physical page allocator



/// The boot memory allocator structure
struct BootMem {
    start: usize,
    size: usize,
}

impl BootMem {


    unsafe fn allocate(&self, _layout: core::alloc::Layout) -> Result<core::ptr::NonNull<[u8]>, ()> {
        todo!()
    }

    unsafe fn deallocate(&self, ptr: core::ptr::NonNull<u8>, layout: core::alloc::Layout) {
        todo!()
    }
}

pub fn init() {

}
