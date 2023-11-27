use crate::{klog, print};
use core::arch::asm;
use bitflags::bitflags;

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

type PDE = u32;
type PTE = u32;

trait BitFlags<T, U> {
    fn set_flags(&mut self, flags : U);
    fn unset_flags(&mut self, flags : U);
}
impl BitFlags<u32, PDEF> for PDE {
    fn set_flags(&mut self, flags : PDEF)
    {
        *self |= flags.bits();
    }
    fn unset_flags(&mut self, flags : PDEF)
    {
        *self &= !flags.bits();
    }
}
impl BitFlags<u32, PTEF> for PDE {
    fn set_flags(&mut self, flags : PTEF)
    {
        *self |= flags.bits();
    }
    fn unset_flags(&mut self, flags : PTEF)
    {
        *self &= !flags.bits();
    }
}

#[repr(align(4096))]
struct AlignedDirectory([PDE; 1024]);
#[repr(align(4096))]
struct AlignedPageTable([PTE; 1024]);

extern "C"
{
    fn load_page_directory(page_directory: *const PDE);
    fn enable_paging();
}

pub fn address_translate()
{
}

pub fn setup_early()
{
    let mut page_directory = AlignedDirectory([0; 1024]);
    let mut page_table = AlignedPageTable([0; 1024]);
    for entry in page_directory.0.iter_mut()
    {
            entry.set_flags(PDEF::Write);
    }
    for (i, entry) in page_table.0.iter_mut().enumerate()
    {
            *entry = (i * 0x1000) as u32;
            entry.set_flags(PTEF::Present | PTEF::Write | PTEF:: User);
    }
    unsafe {
        page_directory.0[0] = (&page_table.0 as *const u32) as u32 & 0xfffff000;
        klog!("Page directory first entry : {:b}", page_directory.0[0]);
        page_directory.0[0].set_flags(PDEF::Present | PDEF::Write | PDEF::User);
        klog!("Page directory first entry : {:b}", page_directory.0[0]);
        // print binary first PDE
        klog!("Page Directory Entry : {:p}", &page_directory);
        klog!("Page Table : {:p}", &page_table);
        klog!("Page directory first entry : {:b}", page_directory.0[0]);
        load_page_directory(&page_directory.0 as *const u32);
        enable_paging();
    }
    klog!("Page Directory Entry : {}", page_directory.0[0]);
    let ptr = 0xdeadbeaf as *mut u8;
    unsafe { *ptr = 42; }
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

