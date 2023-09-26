#![no_main]
#![no_std]

use core::panic::PanicInfo;

mod klib { pub mod mem; }
mod driver { pub mod vga; }
mod arch {
    pub mod x86 {
        pub mod gdt;
        pub mod pic;
        pub mod io;
        pub mod idt;
    }
}
use core::fmt::Write;
use arch::x86::gdt;
use arch::x86::pic;
use arch::x86::idt;
use driver::vga;

use core::arch::asm;

// use core::fmt;


pub static mut VGA_INSTANCE: Option<vga::VGA> = None;

/// This function is called on panic.
#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    unsafe {
        write!(VGA_INSTANCE.as_mut().unwrap(), "Ceci est une panique \n").unwrap(); // TODO log macro
        // print panic 
        write!(VGA_INSTANCE.as_mut().unwrap(), "{}\n", _info).unwrap(); // TODO log macro
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
    unsafe {
        asm!("cli"); // TODO options
        VGA_INSTANCE = Some(vga::VGA::new());
        write!(VGA_INSTANCE.as_mut().unwrap(), "CPU mode: {}\n", get_cpu_mode()).unwrap(); // TODO log macro
        write!(VGA_INSTANCE.as_mut().unwrap(), "SALOPES\n").unwrap(); // TODO log macro
        // let flags = get_flags();
        // if (flags & (1 << 9)) != 0 {
        //     write!(VGA_INSTANCE.as_mut().unwrap(), "Interrupts enabled\n").unwrap();
        // } else {
        //     write!(VGA_INSTANCE.as_mut().unwrap(), "Interrupts disabled\n").unwrap();
        // }


        gdt::load();
        write!(VGA_INSTANCE.as_mut().unwrap(), "Loaded GDT\n").unwrap();

        // let flags = get_flags();
        // if (flags & (1 << 9)) != 0 {
        //     write!(VGA_INSTANCE.as_mut().unwrap(), "Interrupts enabled\n").unwrap();
        // } else {
        //     write!(VGA_INSTANCE.as_mut().unwrap(), "Interrupts disabled\n").unwrap();
        // }
    

        pic::setup(); // TODO error handling in rust 
        write!(VGA_INSTANCE.as_mut().unwrap(), "PIC setup\n").unwrap();
        idt::setup();
        write!(VGA_INSTANCE.as_mut().unwrap(), "IDT setup\n").unwrap();
    }
    // enable interrupts back
    unsafe {
        asm!("sti");
    }
    kloop();
}

#[no_mangle]
pub extern "C" fn _start() -> ! {
    kmain();
}
