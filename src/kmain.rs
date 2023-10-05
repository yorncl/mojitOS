#![no_main]
#![no_std]

use core::panic::PanicInfo;

mod arch;
mod driver;
mod klib;

use arch::x86::gdt;
use arch::x86::pic;
use arch::x86::idt;
use driver::vga;

use core::arch::asm;

// use core::fmt;



/// This function is called on panic.
#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    unsafe {
        klog!("Ceci est une panique \n"); // TODO log macro
        // print panic 
        klog!("{}\n", _info); // TODO log macro
    }
    loop {}
}


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
#[allow(unused_results)] // TODO remove and handle correctly
fn kmain() -> !
{
    vga::io_init(); // TODO this is primitive logging
    klog!("VGA initialized");
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
    }
    kloop();
}

#[no_mangle]
pub extern "C" fn _start() -> ! {
    kmain();
}
