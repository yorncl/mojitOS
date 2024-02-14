pub mod gdt;
pub mod pic;
pub mod io;
pub mod idt;
pub mod paging;
pub mod kstart;

/// x86 page size
pub const PAGE_SIZE : usize = 0x1000;
/// x86 addressable number pages
pub const N_PAGES : usize = 1 << 20;
/// Virtual memory start for kernel
pub const KERNEL_VMEM_START : usize = 0xC0000000;
