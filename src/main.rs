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
    loop {}
}


pub fn get_cpu_mode() -> &'static str {
    let mode: u64;
    unsafe {
        asm!(
            "mov {0}, cr0",
            "and rax, 0x1",
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

#[inline(always)]
fn get_flags() -> u32 {
    let flags: u32;
    unsafe {
        asm!("pushf"," pop {}",out(reg) flags);
    }
    flags
}

#[no_mangle]
fn kloop() -> !
{
    unsafe {
        asm!("sti");
        asm!("int 0x21");
        asm!("cli");
    }
    loop {}
}

#[no_mangle]
fn kmain() -> !
{
    unsafe {
        asm!("cli"); // TODO options
        VGA_INSTANCE = Some(vga::VGA::new());
        write!(VGA_INSTANCE.as_mut().unwrap(), "CPU mode: {}\n", get_cpu_mode()); // TODO log macro
        let flags = get_flags();
        if (flags & (1 << 9)) != 0 {
            write!(VGA_INSTANCE.as_mut().unwrap(), "Interrupts enabled\n");
        } else {
            write!(VGA_INSTANCE.as_mut().unwrap(), "Interrupts disabled\n");
        }


        gdt::load();
        write!(VGA_INSTANCE.as_mut().unwrap(), "Loaded GDT\n");

        let flags = get_flags();
        if (flags & (1 << 9)) != 0 {
            write!(VGA_INSTANCE.as_mut().unwrap(), "Interrupts enabled\n");
        } else {
            write!(VGA_INSTANCE.as_mut().unwrap(), "Interrupts disabled\n");
        }
    

        pic::setup(); // TODO error handling in rust 
        write!(VGA_INSTANCE.as_mut().unwrap(), "PIC setup\n");
        idt::setup();
        write!(VGA_INSTANCE.as_mut().unwrap(), "IDT setup\n");
    }
    kloop();
}

#[no_mangle]
pub extern "C" fn _start() -> ! {
    kmain();
}
