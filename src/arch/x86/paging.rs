use crate::memory::mapper::MapperInterface;
use crate::{klog, print};
use core::ops::DerefMut;
use core::{arch::asm, ops::Deref};
use bitflags::bitflags;
use crate::memory;
use crate::memory::vmm::bump::RawBox;

/// Frame structure for memory system
pub struct Frame(pub usize);

pub static mut PAGING_BASE : RawBox<PageDir> = RawBox {data: 0 as *mut PageDir};

macro_rules!  ROUND_PAGE_UP{
    ($a:expr) => {
           ($a + super::PAGE_SIZE) & !(0xfff as usize)
    };
}

pub (crate) use ROUND_PAGE_UP;

bitflags! {
    #[derive(Copy, Clone)]
    pub struct PDEF : usize {
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

type PDE = usize;
type PTE = usize;

#[repr(C)]
pub struct PageDir
{
        entries: [PDE; 1024]
}
extern "C" {
    static EPD_PHYS: PageDir;
    static EARLY_PAGE_DIRECTORY: PageDir;
}

impl PageDir {
    #[inline(always)]
    pub fn set_entry(&mut self, i: usize, address: usize, flags: usize)
    {
        self.entries[i] = (address & !0xfff) | 3; // TODO convert to flags 
    }
}

impl Default for PageDir
{
    fn default() -> Self
    {
        PageDir {
            entries: [0; 1024]
        }
    }

}

#[repr(C, packed)]
struct PageTable
{
        entries: [PTE; 1024]
}


pub struct X86Mapper {}
pub use X86Mapper as Mapper;
impl X86Mapper {}



#[inline(always)]
fn get_base() -> &'static PageDir
{
    unsafe {PAGING_BASE.deref()}
}



impl memory::mapper::MapperInterface for X86Mapper
{
    fn map_to_virt(f: Frame, address: usize) -> Result<(), ()>
    {
        Ok(())
    }
    fn virt_to_phys(address: usize) -> usize
    {
        let offset = address & 0xfff;
        let pte_offset = ((address >> 12) & 0x3ff) * core::mem::size_of::<u32>(); // TODO refactor types
        let pde_index = address >> 22;
        let special = (0x3ff << 22) | pde_index << 12 | pte_offset;
        let pte : usize;
        unsafe { 
            pte = *(special as *const usize) & !0xfff;
        }
        pte + offset
    }
}

pub fn init_post_jump()
{
        unsafe { 
            // We will use the static early page dir at first TODO should we change it ?
            // TODO oh my god do a macro for symbol I
            // shot myself in the foo multiple times already it hurts so bad
            PAGING_BASE = RawBox::from_ptr(&EARLY_PAGE_DIRECTORY);
            let dir: &mut PageDir = PAGING_BASE.deref_mut();

            // setting the recursive mapping entry at the last entry of the table
            // We lose 4MB of virtual space, but we gain happiness
            // TODO this is very naky, EPD_PHYS is the load address
            dir.set_entry(0x3ff, &EPD_PHYS as *const PageDir as usize, 0);
        }
}

extern "C"
{
    fn load_page_directory(page_directory: *const PageDir);
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

