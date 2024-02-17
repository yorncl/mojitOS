use super::paging;
use crate::driver::vga;
use crate::x86::paging::ROUND_PAGE_UP;
use crate::{klog};
use crate::arch::common::multiboot;
use crate::memory;
use crate::memory::pmm;
use crate::memory::pmm::{Frame, FrameRange};
use crate::memory::vmm;
use crate::memory::vmm::mapper;
use super::PAGE_SIZE;

use alloc::vec;
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
    for entry in memory::phys_mem().regions  {
        klog!("- {entry:?}");
    }

    // This will filter out unusable pages
    klog!("Start init pmm");
    pmm::init(memory::phys_mem());
    // Blocking out the first 4MB as they will always be mapped
    pmm::fill_range(FrameRange{start: Frame(0), size: (kend - super::KERNEL_OFFSET) / super::PAGE_SIZE});
    klog!("End init pmm");

    paging::init_post_jump();
    klog!("virt to phys : {:?}", mapper::virt_to_phys_kernel(0x1000));
    klog!("virt to phys : {:?}", mapper::virt_to_phys_kernel(0xC0001000));

    // range mapping test
    // let mut a = 0xF0001000;
    // a += 100;
    // klog!("0x{a:x} access : {:?}", mapper::virt_to_phys_kernel(a));
    // a += 0x1000;
    // klog!("0x{a:x} access : {:?}", mapper::virt_to_phys_kernel(a));

    // a = 0xF0001000;
    // let range = pmm::alloc_contiguous_pages(2).unwrap();
    // mapper::map_range_kernel(range, a).expect("Could not test map");

    // a += 100;
    // klog!("0x{a:x} access : {:?}", mapper::virt_to_phys_kernel(a));
    // a += 0x1000;
    // klog!("0x{a:x} access : {:?}", mapper::virt_to_phys_kernel(a));

    // Sets up the virtual memory manager
    let memstart = ROUND_PAGE_UP!(kend);
    vmm::init(memstart, super::KERNEL_PAGE_TABLES_START - kend);

    let vec_test = vec![1;100];
    let vec_test2 = vec![1;100];
    loop{}
    let s = alloc::string::String::from("Bonjour tout le monde");


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

    // crate::kmain();    
}
