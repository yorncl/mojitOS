pub mod gdt;
pub mod pic;
pub mod io;
pub mod idt;
pub mod paging;
pub mod cpuid;
mod util;
mod apic;
mod acpi;
mod iomem;
pub mod kstart;

use crate::MB;

/// x86 page size
pub const PAGE_SIZE : usize = 0x1000;
/// x86 addressable number pages
pub const N_PAGES : usize = 1 << 20;
/// Virtual memory start for kernel
pub const KERNEL_OFFSET : usize = 0xC0000000;

/// Start of the 4MB block for page tables
/// Last 4 MB of virtual space TODO I don't know where to put it
pub const KERNEL_PAGE_TABLES_START : usize = 0xff400000;
/// Size of page table block
pub const KERNEL_PAGE_TABLES_SIZE : usize = MB!(4);

pub const KERNEL_IOMM_START : usize = 0xff800000;
pub const KERNEL_IOMM_SIZE : usize = MB!(1);


/*

Virtual memory map


------------------- 0x0

Userspace

------------------- 0xC0000000

Kernel Main memory

------------------- 0xff400000

Kernel page tables (4MB)

------------------- 0xff800000
IO remap area (1MB)

------------------- 0xffc00000

The last 4MB are reserved in the Page Directory to achieve recursive mapping

------------------- 0xffffffff

*/
