pub mod gdt;
pub mod pic;
pub mod io;
pub mod idt;
pub mod paging;
pub mod kstart;

use crate::MB;

/// x86 page size
pub const PAGE_SIZE : usize = 0x1000;
/// x86 addressable number pages
pub const N_PAGES : usize = 1 << 20;
/// Virtual memory start for kernel
pub const KERNEL_HEAP_START : usize = 0xC0000000;
/// Start of the 4MB block for page tables
/// Last 4 MB of virtual space TODO I don't know where to put it
pub const KERNEL_PAGE_TABLES_START : usize = 0xff800000;
/// Common use virtual memory size for kernel
pub const KERNEL_HEAP_SIZE : usize = KERNEL_PAGE_TABLES_START - KERNEL_HEAP_START;
/// Size of page table block
pub const KERNEL_PAGE_TABLES_SIZE : usize = MB!(4);
