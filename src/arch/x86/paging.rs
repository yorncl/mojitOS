use crate::{klog, print};
use core::arch::asm;

#[repr(u32)]
#[allow(dead_code)]
enum PDEFlag // aligned for 4kb
{
    Present = 1,
    Write = 1 << 1,
    User = 1 << 2,
    WriteThrough = 1 << 3,
    CacheDisable = 1 << 4,
    Accessed = 1 << 5,
    Available = 1 << 6,
    PageSize = 1 << 7,
}

type PDE = u32;

trait BitSet<T>
{
    fn set(&mut self, flag: T);
    fn unset(&mut self, flag: T);
}

impl BitSet<PDEFlag> for PDE
{
    fn set(&mut self, flag: PDEFlag)
    {
        *self |= flag as u32;
    }

    fn unset(&mut self, flag: PDEFlag)
    {
        *self &= !(flag as u32);
    }
}


#[repr(u32)]
#[allow(dead_code)]
enum PTEFlag
{
    Present = 1,
    Write = 1 << 1,
    User = 1 << 2,
    WriteThrough = 1 << 3,
    CacheDisable = 1 << 4,
    Accessed = 1 << 5,
    Dirty = 1 << 6,
    PageAttribute = 1 << 7,
    Global = 1 << 8
}

type PTE = u32;

impl BitSet<PTEFlag> for PTE
{
    fn set(&mut self, flag: PTEFlag)
    {
        *self |= flag as u32;
    }

    fn unset(&mut self, flag: PTEFlag)
    {
        *self &= !(flag as u32);
    }
}

#[repr(align(4096))]
struct AlignedDirectory([PDE; 1024]);
#[repr(align(4096))]
struct AlignedPageTable([PDE; 1024]);

// fn enable_paging()
// {
// }

extern "C"
{
    #[no_mangle]
    fn load_page_directory(page_directory: *const PDE);
    #[no_mangle]
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
            entry.set(PDEFlag::Write);
    }
    for (i, entry) in page_table.0.iter_mut().enumerate()
    {
            *entry = (i * 0x1000) as u32;
            entry.set(PTEFlag::Present);
            entry.set(PTEFlag::Write);
            entry.set(PTEFlag::User);
            // klog!("Page Table Entry : {:b}", entry);
    }
    unsafe {
        page_directory.0[0] = (&page_table.0 as *const u32) as u32 & 0xfffff000;
        page_directory.0[0].set(PDEFlag::Present);
        page_directory.0[0].set(PDEFlag::Write);
        page_directory.0[0].set(PDEFlag::User);
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

mod PF
{
    pub const P : u32 = 1 << 0;
    pub const W : u32 = 1 << 1;
    pub const U : u32 = 1 << 2;
    pub const R : u32 = 1 << 3;
    pub const I : u32 = 1 << 4;
    pub const PK : u32 = 1 << 5;
    pub const SS : u32 = 1 << 6;
    pub const SGX : u32 = 1 << 7;
}
use PF::*;

pub fn page_fault_handler(instrction_pointer: u32, code: u32)
{
    let address : u32;
    klog!("PAGE FAULT EXCEPTION");
    unsafe {asm!("mov {0}, cr2", out(reg) address);}
    klog!("Virtual address : {:p}", (address as *const u32));
    print!("Error code: "); // TODO reformat in the future
    print!("{} ", if code & PF::P != 0 {"PAGE_PROTECTION"} else {"PAGE_NOT_PRESENT"});
    print!("{} ", if code & PF::W != 0 {"WRITE"} else {"READ"});
    if code & PF::U != 0 { print!("CPL_3 ") };
    if code & PF::R != 0 { print!("RESERVED_WRITE_BITS ") };
    if code & PF::I != 0 { print!("INSTRUCTION_FETCH ") };
    if code & PF::PK != 0 { print!("KEY_PROTECTION ") };
    if code & PF::SS != 0 { print!("SHADOW STACK ") };
    if code & PF::SGX != 0 { print!("SGX_VIOLATION ") };
    print!("\n");
    loop{}
}

