use super::paging;
use crate::driver::vga;
use crate::x86::apic;
use crate::x86::paging::ROUND_PAGE_UP;
use crate::{klog, dbg};
use crate::arch::common::multiboot;
use crate::memory;
use crate::memory::pmm;
use crate::memory::pmm::{Frame, FrameRange};
use crate::memory::vmm;
use super::PAGE_SIZE;

use crate::driver;

use super::idt;
use super::gdt;


use super::cpuid;
use super::pic;
use super::acpi;

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

    // Cpu features requirements
    cpuid::init();
    klog!("CPU vendor: {}", cpuid::vendor());
    if !cpuid::check_local_apic() {
        panic!("APIC needed!")
    }
    if !cpuid::check_rdmsr() {
        panic!("RDMSR disabled!")
    }
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
    // Blocking out the first 4MB as they are already mapped and always will be
    pmm::fill_range(FrameRange{start: Frame(0), size: (kend - super::KERNEL_OFFSET) / super::PAGE_SIZE});
    klog!("End init pmm");

    klog!("Setup paging post jump");
    paging::init_post_jump();

    klog!("Setting up the memory manager");
    // Sets up the virtual memory manager
    let memstart = ROUND_PAGE_UP!(kend);
    vmm::init(memstart, super::KERNEL_PAGE_TABLES_START - kend);

    super::disable_interrupts();
    klog!("Disabling PIC");
    // pic::setup();
    pic::disable();

    // TODO this sets up the APIC behind the scene, make it more transparent
    klog!("Setup ACPI");
    acpi::init().unwrap();

    // klog!("Setup APIC timer");
    apic::timer::init();

    gdt::load();
    klog!("GDT loaded");
    idt::setup();
    klog!("IDT setup");

    // PS/2 keyboard
    driver::kbd::init().unwrap();

    crate::kmain();
}
