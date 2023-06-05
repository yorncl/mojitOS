use core::arch::global_asm;


pub mod x86 {
    pub mod gdt;
    pub mod pic;
    pub mod io;
    pub mod idt;
}
