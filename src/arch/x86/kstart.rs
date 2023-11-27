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

#[no_mangle]
pub unsafe extern "C" fn kstart(magic: u32, mboot: *const u32) -> !
{
    vga::io_init(); // TODO this is primitive logging, maybe we need to wait for the whole memory
                    // to setup
    klog!("VGA initialized");
    // klog!("Multiboot: magic({:x}) mboot({:p})", magic, mboot);
    // parse_mboot_info(mboot);
    unsafe {
        asm!("cli");
        klog!("CPU mode: {}", get_cpu_mode());
        gdt::load();
        klog!("GDT loaded");
        pic::setup(); // TODO error handling in rust 
        klog!("PIC setup");
        idt::setup();
        klog!("IDT setup");
        asm!("sti");
        pmm::init();
        vmm::init();
        loop{}
        paging::setup_early();
    }
    crate::kmain();    
}
