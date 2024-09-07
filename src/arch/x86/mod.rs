pub mod context;
pub mod cpu;
pub mod gdt;
pub mod idt;
pub mod io;
pub mod irq;
pub mod kstart;
pub mod lock;
pub mod paging;
pub mod pic;
pub mod timer;

mod acpi;
mod apic;
// mod bootmem;
mod iomem;
mod util;

/*

Physical memory map

------------------- 0x0

DMA

------------------- 0x100000

Kernel Main memory

...

*/

/*

Virtual memory map


------------------- 0x0

Userspace

------------------- 0xC0000000

Linear mapping of physical memory
Around 800meg (0x30000000)

------------------- 0xf0000000

Temporary mappings

*/

/// Start of the linear identity mapping
pub const KERNEL_LINEAR_START: usize = 0xC0000000;
/// Virtual memory mapping area
pub const KERNEL_TEMP_START: usize = 0xf0000000;

pub const PAGE_SIZE: usize = 0x1000;
pub const N_PAGES: usize = 1 << 20;

// TODO move somewhere else
use core::arch::asm;
pub fn disable_interrupts() {
    unsafe { asm!("cli") };
}
pub fn enable_interrupts() {
    unsafe { asm!("sti") };
}
