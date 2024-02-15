use crate::{klog, print};
use core::ops::{Deref, DerefMut};
use core::arch::asm;
use bitflags::bitflags;
use crate::memory;
use crate::utils::rawbox::RawBox;
use crate::memory::pmm;
use crate::memory::pmm::{Frame, FrameRange};
use super::PAGE_SIZE;
use crate::memory::vmm::mapper;

pub static mut MAPPER : RawBox<PageDir> = RawBox {data: 0 as *mut PageDir};

macro_rules!  ROUND_PAGE_UP{
    ($a:expr) => {
           ($a + super::PAGE_SIZE) & !(0xfff as usize)
    };
}

extern "C" {
    static EPD_PHYS: PageDir;
    static EARLY_PAGE_DIRECTORY: PageDir;
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

impl PageDir {
    #[inline(always)]
    pub fn set_entry(&mut self, i: usize, address: usize, flags: usize)
    {
        self.entries[i] = (address & !0xfff) | flags; // TODO convert to flags 
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

fn flush_tlb()
{
    unsafe {
        asm!("push eax",
        "mov eax, cr3",
        "mov cr3, eax",
        "pop eax");
    }
}

#[repr(C, packed)]
struct PageTable
{
        entries: [PTE; 1024]
}

#[inline(always)]
pub fn kernel_mapper() -> &'static mut PageDir
{
    unsafe {MAPPER.deref_mut()}
}

macro_rules! pde_index {
    ($a: expr) => {
       $a >> 22
    };
}

macro_rules! pte_index {
    ($a: expr) => {
       $a >> 12 & (2 << 10 - 1)
    };
}

macro_rules! offset {
    ($a: expr) => {
       $a & (2 << 12 - 1)
    };
}

impl mapper::MapperInterface for PageDir
{

    fn map_single(&mut self, f: Frame, address: usize) -> Result<(), ()> {
        
        flush_tlb();
        Err(())
    }

    fn map_range(&mut self, r: FrameRange, address: usize) -> Result<(), ()> {
        // TODO assert aligned to page 

        let phys_start = r.start.0 * PAGE_SIZE; // TODO more generic api
        for i in 0..r.size {

        }
        flush_tlb();
        Err(())
    }

    fn virt_to_phys(&mut self, address: usize) -> Option<usize>
    {
        let pde_index = address >> 22;
        if self.entries[pde_index] == 0 {
            return None;
        }
        let offset = address & 0xfff;
        let pte_offset = ((address >> 12) & 0x3ff) * core::mem::size_of::<u32>(); // TODO refactor types

        let special = (0x3ff << 22) | pde_index << 12 | pte_offset;
        let pte : usize;
        unsafe { 
            pte = *(special as *const usize) & !0xfff;
        }
        Some(pte + offset)
    }

}

// TODO might put this in the assembly
static mut KERNEL_PT_TEMP : [u32; 1024] = [0; 1024];

pub fn init_post_jump()
{
        unsafe { 
            // We will use the static early page dir at first TODO should we change it ?
            // TODO oh my god do a macro for getting symbols's address I
            // shot myself in the foot multiple times already it hurts so bad
            MAPPER = RawBox::from_ptr(&EARLY_PAGE_DIRECTORY);
            let dir: &mut PageDir = MAPPER.deref_mut();

            // setting the recursive mapping entry at the last entry of the table
            // We lose 4MB of virtual space, but we gain happiness
            // TODO this is very naky, EPD_PHYS is the load address
            dir.set_entry(0x3ff, &EPD_PHYS as *const PageDir as usize, 3);


            // remove identity mapping
            dir.set_entry(0, 0, 0);

            // Allocating 4MB in high memory to store the kernel page tables
            assert!(super::KERNEL_PAGE_TABLES_START >> 22 == 0x3fe);
            let address = mapper::virt_to_phys_kernel(&KERNEL_PT_TEMP as *const u32 as usize).unwrap();
            dir.set_entry(0x3fe, address, 3);
            
            // flush the tlb so we can map in the new table
            flush_tlb();

            let range = pmm::alloc_contiguous_pages(1024).expect("Cannot allocate page tables memory space");
            mapper::map_range_kernel(range, super::KERNEL_PAGE_TABLES_START).expect("Cannot map page tables memory space");
        }
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

