use super::gdt;
use super::pic;
use super::idt;
use super::paging;
use crate::driver::vga;
use crate::arch::common::pmm;
use super::vmm;
use crate::klog;
use crate::arch::common::multiboot::*;

use core::arch::asm;

pub fn get_cpu_mode() -> &'static str {
    let mode: u32;
    unsafe {
        asm!(
            "mov {0}, cr0",
            "and eax, 0x1",
            out(reg) mode,
            options(nostack, preserves_flags)
        );
    }

    if mode == 0 {
        "real mode"
    } else if mode == 1 {
        "protected mode"
    } 
    else {
        "wtf"
    }
}

extern "C" {
    static kernel_image_start : u32;
    static kernel_image_end : u32;
}

#[no_mangle]
pub extern "C" fn kstart(magic: u32, mboot: *const u32) -> !
{
    vga::io_init(); // TODO this is primitive logging, maybe we need to wait for the whole memory
                    // to setup
    klog!("VGA initialized");

    loop{}
    // klog!("Multiboot: magic({:x}) mboot({:p})", magic, mboot);
    parse_mboot_info(mboot);
    unsafe {
        klog!("This is the kernel's start {:p}", &kernel_image_start);
        klog!("This is the kernel's start {:x}", &kernel_image_start);
        klog!("This is the kernel's end {:p}", &kernel_image_end);
    }
    // klog!("This is reload_segments's address {:p}", reload_segments as *const());
    unsafe { asm!("cli"); }
    klog!("CPU mode: {}", get_cpu_mode());
    gdt::load();
    klog!("GDT loaded");
    pic::setup(); // TODO error handling in rust 
    klog!("PIC setup");
    idt::setup();
    klog!("IDT setup");
    unsafe { asm!("sti"); }


    // Physical memory manager
    // Setting up the necessary data structures
    pmm::init();

    // Setting up the kernel virtual memory
    // This functions will setup all the necessary structures for page management
    // It will setup the PMM to reflect the current state of the memory
    vmm::init();

    // Once everything is ready, enable the paging
    paging::enable();
    // Memory management should be setup from there

    crate::kmain();    
}
