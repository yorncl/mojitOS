use super::gdt;
use super::pic;
use super::idt;
use super::paging;
use crate::driver::vga;
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

// #[inline(always)]
// fn get_flags() -> u32 {
//     let flags: u32;
//     unsafe {
//         write!(VGA_INSTANCE.as_mut().unwrap(), "BEFORE\n").unwrap(); // TODO log macro
//         asm!("pushf"," pop {}",out(reg) flags, options(nostack, preserves_flags)); // TODO this
//                                                                                    // crashes
//         write!(VGA_INSTANCE.as_mut().unwrap(), "AFTER\n").unwrap(); // TODO log macro
//     }
//     flags
// }

#[no_mangle]
fn kloop() -> !
{
    loop {
    }
}

#[no_mangle]
pub unsafe extern "C" fn kstart(magic: u32, mboot: *const u32) -> !
{
    vga::io_init(); // TODO this is primitive logging, maybe we need to wait for the whole memory
                    // to setup
    klog!("VGA initialized");
    klog!("Multiboot: magic({:x}) mboot({:p})", magic, mboot);
    parse_mboot_info(mboot);
    loop{}
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
        paging::setup();
    }
    crate::kmain();    
}
