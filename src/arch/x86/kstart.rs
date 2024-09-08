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
use super::cpu;
use super::pic;
use super::acpi;
use super::iomem;

extern "C" {
    /// Defined in linker file
    static kernel_image_start : u32;
    /// Defined in linker file
    static kernel_image_end : u32;
}

/// Entrypoint post boot initialization
/// At this point the first 4MB of physical memory, containing the kernel and some DMA areas, are mapped at two places
/// We already are in higher half
#[no_mangle]
pub extern "C" fn kstart(_magic: u32, mboot: *const u32) -> !
{
    super::disable_interrupts();
    // early vga logging
    vga::io_init();
    klog!("Early boot setup...");

    // Cpu features requirements
    cpu::cpuid_fetch();
    if !cpu::has_feature(cpu::Flags::MSR) {
        panic!("Kernel require rdmsr")
    }
    dbg!("CPU vendor: {}", cpu::vendor());
    if !cpu::has_feature(cpu::Flags::APIC) {
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

    // TODO should it be there
    dbg!("Loading GDT");
    gdt::load();
    dbg!("Loading IDT");
    idt::setup();

    paging::cleanup_post_jump();
    // This will filter out unusable pages
    dbg!("Initializing physical memory manager");
    pmm::init(memory::phys_mem());
    // Blocking out the first 4MB as they are already mapped and always will be
    // setting the first 4MB of PMM bitmap TODO api seems dirty
    pmm::fill_range(FrameRange{start: Frame(0), size: (kend - super::KERNEL_LINEAR_START) / super::PAGE_SIZE});

    loop{}
    dbg!("Initializing kernel allocator");
    // Sets up the virtual memory manager
    let memstart = ROUND_PAGE_UP!(kend);
    vmm::init(memstart, todo!());

    dbg!("Disabling PIC");
    pic::disable();

    dbg!("Setup iomem");
    // TODO fix ugly, remove altogether ?
    let _ = iomem::init();
    // TODO this sets up the APIC behind the scene, make it more transparent
    dbg!("Reading ACPI information");
    acpi::init().unwrap();

    dbg!("Setup APIC timer");
    apic::timer::init();

    loop{}
    crate::kmain();
}
