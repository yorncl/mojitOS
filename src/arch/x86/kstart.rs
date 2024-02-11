use super::paging;
use crate::driver::vga;
use crate::{klog, memory};
use crate::arch::common::multiboot;
use crate::memory::pmm::PageManager;
use crate::memory::vmm::bump::{Bump, RawBox};
use crate::memory::{BUMP_ALLOCATOR};


// pub fn get_cpu_mode() -> &'static str {
//     let mode: u32;
//     unsafe {
//         asm!(
//             "mov {0}, cr0",
//             "and eax, 0x1",
//             out(reg) mode,
//             options(nostack, preserves_flags)
//         );
//     }

//     if mode == 0 {
//         "real mode"
//     } else if mode == 1 {
//         "protected mode"
//     } 
//     else {
//         "wtf"
//     }
// }

extern "C" {
    static kernel_image_start : u32;
    static kernel_image_end : u32;
}

/// Entrypoint post boot initialization
/// At this point the first 4MB of physical memory containing the kernel are mapped at two places
#[no_mangle]
pub extern "C" fn kstart(magic: u32, mboot: *const u32) -> !
{
    // early vga logging
    vga::io_init();
    klog!("VGA initialized");

    let kstart: usize;
    let kend: usize;
    unsafe {
        klog!("Kernel start {:p}", &kernel_image_start);
        klog!("Kernel end {:p}", &kernel_image_end);
        kstart = &kernel_image_start as *const u32 as usize;
        kend = &kernel_image_end as *const u32 as usize;
    }
    // TODO should I calulate this before jumping to kstart, as it might require identity mapping more pages
    // at the start ?
    let ksize = (kend - kstart)/1024; 
    klog!("Kernel size : {}KB", ksize);
    klog!("Multiboot: magic({:x}) mboot({:p})", magic, mboot);

    // setting the first 4MB of PMM bitmap TODO api seems dirty
    unsafe {
        memory::pmm::PMM.fill_range(0, 4096);
        klog!("alloc page i = {}", memory::pmm::PMM.alloc_page().unwrap());
        klog!("alloc page i = {}", memory::pmm::PMM.alloc_page().unwrap());
        klog!("alloc page i = {}", memory::pmm::PMM.alloc_page().unwrap());
    }

    // Figuring out the physical memory layout
    // Here we assume the kernel is booted using multiboot
    use multiboot::MbootError;
    match multiboot::parse_mboot_info(mboot)
    {
            Err(MbootError::InvalidFlags) => {panic!("Multiboot flags malformed")},
            Err(MbootError::NoMemoryMap) => {panic!("No memory map")}, // TODO BIOS functions ?
            Ok(()) => (),
    }

    klog!("Physical Memory regions:");
    for i in 0..10  {
        let entry;
        unsafe {
            entry = memory::PHYS_MEM[i];
            klog!("- {entry:?}");
        };
    }

    let memstart = paging::ROUND_PAGE_UP!(kend);
    klog!("Start of free mem: 0x{:x}", paging::ROUND_PAGE_UP!(kend));
    unsafe { 
        BUMP_ALLOCATOR = Bump{start: memstart, size: 0};
    }


    struct ChadStruct {
        chad: u64,
        chad_bis: u64
    }

    let a = RawBox::new(69);
    let chad = RawBox::new(ChadStruct{chad: 69, chad_bis: 420});
    let b = RawBox::new(420);
    klog!("{}", a);
    klog!("{}", chad);
    klog!("{}", b);

    loop{}

    // bump.allocate(n);
    // loop {}
    // // klog!("This is reload_segments's address {:p}", reload_segments as *const());
    // unsafe { asm!("cli"); }
    // klog!("CPU mode: {}", get_cpu_mode());
    // gdt::load();
    // klog!("GDT loaded");
    // pic::setup(); // TODO error handling in rust 
    // klog!("PIC setup");
    // idt::setup();
    // klog!("IDT setup");
    // unsafe { asm!("sti"); }


    // // Physical memory manager
    // // Setting up the necessary data structures
    // pmm::init();

    // // Setting up the kernel virtual memory
    // // This functions will setup all the necessary structures for page management
    // // It will setup the PMM to reflect the current state of the memory
    // vmm::init();

    // // Once everything is ready, enable the paging
    // paging::enable();
    // // Memory management should be setup from there

    // crate::kmain();    
}
