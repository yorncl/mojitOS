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
------------------- 0xffbfffff

*/

/// Start of the linear identity mapping of the physical memory
pub const KERNEL_LINEAR_START: usize = 0xC0000000;
/// Virtual memory mapping area, no fixed mappings
/// The higher mapping will be swapped as needed, CF linux x86 memory model
pub const KERNEL_TEMP_START: usize = 0xf0000000;
// The last 4MB of virtual space will be used to remap some addresses for IO devices (eg. IOAPIC) 
pub const KERNEL_IO_REMAP: usize = 0xffbfffff;

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
