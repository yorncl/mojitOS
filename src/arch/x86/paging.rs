use crate::{klog, print};
use core::arch::asm;
use bitflags::bitflags;


/// Frame structure for memory system
pub struct Frame(pub usize);

macro_rules!  ROUND_PAGE_UP{
    ($a:expr) => {
           ($a + super::PAGE_SIZE) & !(0xfff as usize)
    };
}

pub (crate) use ROUND_PAGE_UP;

bitflags! {
    #[derive(Copy, Clone)]
    pub struct PDEF : u32 {
        const Present = 1;
        const Write = 1 << 1;
        const User = 1 << 2;
        const WriteThrough = 1 << 3;
        const CacheDisable = 1 << 4;
        const Accessed = 1 << 5;
        const Available = 1 << 6;
        const PageSize = 1 << 7;
        const _ = !0;
    }
}

bitflags! {
    #[derive(Copy, Clone)]
    pub struct PTEF : u32 {
        const Present = 1;
        const Write = 1 << 1;
        const User = 1 << 2;
        const WriteThrough = 1 << 3;
        const CacheDisable = 1 << 4;
        const Accessed = 1 << 5;
        const Dirty = 1 << 6;
        const PageAttribute = 1 << 7;
        const Global = 1 << 8;
        const _ = !0;
    }
}

bitflags! {
    #[derive(Copy, Clone)]
    pub struct PF : u32
    {
        const P = 1 << 0;
        const W = 1 << 1;
        const U = 1 << 2;
        const R = 1 << 3;
        const I = 1 << 4;
        const PK = 1 << 5;
        const SS = 1 << 6;
        const SGX = 1 << 7;
        const _ = !0;
    }
}

type PDE = u32;
type PTE = u32;

extern "C"
{
    fn load_page_directory(page_directory: *const PDE);
    fn enable_paging();
}


// TODO is this code archiecture specific ?
pub fn page_fault_handler(instruction_pointer: u32, code: u32)
{
    let address : u32;
    klog!("PAGE FAULT EXCEPTION");
    unsafe {asm!("mov {0}, cr2", out(reg) address);}
    klog!("Virtual address : {:p}", (address as *const u32));
    print!("Error code: "); // TODO reformat in the future
    let flags = PF::from_bits(code).unwrap();
    print!("{} ", if flags.contains(PF::P) {"PAGE_PROTECTION"} else {"PAGE_NOT_PRESENT"});
    print!("{} ", if flags.contains(PF::W) {"WRITE"} else {"READ"});
    if flags.contains(PF::U) { print!("CPL_3 ") };
    if flags.contains(PF::R) { print!("RESERVED_WRITE_BITS ") };
    if flags.contains(PF::I) { print!("INSTRUCTION_FETCH ") };
    if flags.contains(PF::PK) { print!("KEY_PROTECTION ") };
    if flags.contains(PF::SS) { print!("SHADOW STACK ") };
    if flags.contains(PF::SGX) { print!("SGX_VIOLATION ") };
    print!("\n");
    loop{}
}

