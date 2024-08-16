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
use super::idt;
use super::gdt;
use super::cpuid;
use super::pic;
use super::acpi;

extern "C" {
    /// Defined in linker file
    static kernel_image_start : u32;
    /// Defined in linker file
    static kernel_image_end : u32;
}

/// Entrypoint post boot initialization
/// At this point the first 4MB of physical memory, containing the kernel and some DMA areas, are mapped at two places
#[no_mangle]
pub extern "C" fn kstart(_magic: u32, mboot: *const u32) -> !
{
    // early vga logging
    vga::io_init();
    dbg!("VGA initialized");

    // Cpu features requirements
    cpuid::init();
    if !cpuid::check_rdmsr() {
        panic!("Kernel require rdmsr")
    }

    dbg!("CPU vendor: {}", cpuid::vendor());
    if !cpuid::check_local_apic() {
        panic!("APIC needed!")
    }

    // Calculating the kernel image size
    let kstart: usize;
    let kend: usize;
    unsafe {
        dbg!("Kernel start {:p}", &kernel_image_start);
        dbg!("Kernel end {:p}", &kernel_image_end);
        kstart = &kernel_image_start as *const u32 as usize;
        kend = &kernel_image_end as *const u32 as usize;
    }
    let ksize = (kend - kstart)/1024; 
    dbg!("Kernel size : {}KB", ksize);
    // TODO we need better early mapping if the kernel is too big
    assert!(ksize < 3 * 1024 * 1024);

    // Figuring out the physical memory layout
    // Here we assume the kernel is booted using multiboot
    if multiboot::parse_mboot_info(mboot).is_err() {
            panic!("Multiboot config error");
    }

    dbg!("Physical Memory regions:");
    for _entry in memory::phys_mem().regions  {
        dbg!("- {:?}", _entry);
    }

    // This will filter out unusable pages
    klog!("Initializing physical memory manager");
    pmm::init(memory::phys_mem());
    // Blocking out the first 4MB as they are already mapped and always will be
    // setting the first 4MB of PMM bitmap TODO api seems dirty
    pmm::fill_range(FrameRange{start: Frame(0), size: (kend - super::KERNEL_OFFSET) / super::PAGE_SIZE});

    klog!("Setup paging post jump");
    paging::init_post_jump();

    klog!("Initializing kernel allocator");
    // Sets up the virtual memory manager
    let memstart = ROUND_PAGE_UP!(kend);
    vmm::init(memstart, super::KERNEL_PAGE_TABLES_START - kend);

    super::disable_interrupts();
    klog!("Disabling PIC");
    // pic::setup();
    pic::disable();

    // TODO this sets up the APIC behind the scene, make it more transparent
    klog!("Reading ACPI information");
    acpi::init().unwrap();

    // klog!("Setup APIC timer");
    apic::timer::init();

    klog!("Loading GDT");
    gdt::load();
    klog!("Loading IDT");
    idt::setup();

    crate::kmain();
}
