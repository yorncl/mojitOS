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

pub use paging::Frame;


use core::fmt;
// TODO move ? kind of ugly
impl fmt::Display for paging::Frame
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "({}, {})", self.0, self.0)
    }
}
