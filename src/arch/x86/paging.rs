use super::{KERNEL_LINEAR_START, PAGE_SIZE};
use crate::memory::pmm::{Frame, FrameRange};
use crate::memory::vmm::mapper;
use crate::utils::rawbox::RawBox;
use crate::MB;
use crate::{dbg, klog, kprint};
use bitflags::{bitflags, Flags};
use core::arch::asm;
use core::ops::DerefMut;
use crate::error::{Result, codes::*};

// pub static mut MAPPER: RawBox<PageDir> = RawBox {
//     data: 0 as *mut PageDir,
// };

#[macro_export]
macro_rules! ROUND_PAGE_UP {
    ($a:expr) => {
        ($a + PAGE_SIZE) & !(0xfff as usize)
    };
}

#[macro_export]
macro_rules! ROUND_PAGE_DOWN {
    ($a:expr) => {
        ROUND_PAGE_UP!($a) - PAGE_SIZE
    };
}

macro_rules! pde_index {
    ($a: expr) => {
        ($a >> 22)
    };
}

macro_rules! pte_index {
    ($a: expr) => {
        ($a >> 12) & ((1 << 10) - 1)
    };
}

macro_rules! offset {
    ($a: expr) => {
        ($a & ((1 << 12) - 1))
    };
}

macro_rules! is_page_aligned {
    ($a: expr) => {
        ($a & (PAGE_SIZE - 1) == 0)
    };
}

#[no_mangle]
static mut KERNEL_PD: PageDir = PageDir::new();

pub(crate) use ROUND_PAGE_UP;

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
        const Global = 1 << 8;
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

// type PDE = u32;
type PTE = u32;
#[derive(PartialEq, Clone, Copy)]
struct PDE(u32);

// Je suis dans ma paranoia
const _: [u8; 4] = [0; core::mem::size_of::<PDE>()];
const _: [u8; 1024 * 4] = [0; core::mem::size_of::<PageDir>()];

impl PDE {
    const fn new(address: u32, flags: PDEF) -> PDE {
        PDE(address | flags.bits())
    }
}

#[repr(C, align(4096))]
pub struct PageDir {
    entries: [PDE; 1024],
}

impl PageDir {
    const fn new() -> Self {
        PageDir {
            entries: [PDE(0); 1024],
        }
    }

    pub fn dump_dbg(&self) {
        dbg!("Dumping page directory");
        for i in 0..1024 {
            if self.entries[i].0 != 0 {
                dbg!(
                    " pd entry {} -> {:x} ({:x} - {:x})",
                    i,
                    self.entries[i].0,
                    ((i as usize) << 22),
                    ((i + 1 as usize) << 22)
                );
                let special = (0x3ff << 22) | i << 12;
                dbg!("  Page table");
                let pt: &mut PageTable = unsafe { &mut *(special as *mut PageTable) };

                let mut count = -1;
                let mut prev: u32 = { pt.entries[0] };
                let mut pdelta: i64 = 0;
                // continue;

                for j in 0..1024 {
                    let entry = { pt.entries[j] };

                    let delta = entry as i64 - prev as i64;
                    if delta == pdelta {
                        count += 1;
                    } else {
                        if count >= 3 {
                            dbg!("   ... x{} delta {:x}", count - 3, pdelta);
                        }
                        count = 0;
                        pdelta = delta;
                    }
                    if count < 3 {
                        dbg!(
                            "   pt entry {} -> {:x} ({:x})",
                            j,
                            entry,
                            ((i as usize) << 22 | (j as usize) << 12)
                        );
                    }
                    prev = entry;
                }
            }
        }
    }
}

fn flush_tlb() {
    unsafe {
        asm!("push eax", "mov eax, cr3", "mov cr3, eax", "pop eax");
    }
}

#[repr(C, packed)]
struct PageTable {
    pub entries: [PTE; 1024],
}

#[inline(always)]
pub fn kernel_mapper() -> &'static mut PageDir {
    // TODO clean up
    unsafe { &mut KERNEL_PD }
}

impl mapper::MapperInterface for PageDir {
    /// Map a single physical frame to a virtual address
    fn map_single(&mut self, f: Frame, address: usize) -> Result<()> {
        todo!();
        // let phys_address = f.0 * PAGE_SIZE;
        // dbg!("Mapping virt {:x} to phys {:x}", address, phys_address);
        // if let Some(mapped) = self.virt_to_phys(address) {
        //     dbg!(
        //         "Mapping already mapped address {:x}, currently mapped to {:x} ",
        //         address,
        //         mapped
        //     );
        //     panic!("Mapping already mapped page");
        // }
        // if !is_page_aligned!(address) {
        //     return Err(());
        // }
        // let pde_index = pde_index!(address);
        // let pt = unsafe { &mut (*get_kernel_pt(pde_index)) };
        // if self.entries[pde_index] == 0 {
        //     self.entries[pde_index] =
        //         self.virt_to_phys(pt as *const PageTable as usize).unwrap() | 3;
        //     // Avoid the page table being filled with junk
        //     memset(pt as *const PageTable as *mut c_void, 0, 4096);
        // }
        // pt.entries[pte_index!(address)] = phys_address | 3;

        // // self.dump_dbg();
        // flush_tlb();
        // Ok(())
    }

    /// Map a single page and release its physical frame
    fn unmap_single(&mut self, address: usize) -> Result<()> {
        if !is_page_aligned!(address) {
            return Err(EFAULT);
        }
        dbg!("Unmapping a single frame");
        // Making sure that the page is mapped
        let address = self.virt_to_phys(address).ok_or(EFAULT)?;

        // free the entry in the page directory
        // Use recursive mapping to get to the corresponding page table
        // The offset is to 0 so we have the start of the table
        let special = (0x3ff << 22) | pde_index!(address) << 12;
        let pt: &mut PageTable = unsafe { &mut *(special as *mut PageTable) };

        todo!();

        // Release the physical frame
        flush_tlb();
        Ok(())
    }

    /// Unmap multiple pages and release their physical frames
    fn unmap_range(&mut self, address: usize, npages: usize) -> Result<()> {
        let mut ptr = address;
        for _ in 0..npages {
            self.unmap_single(ptr).unwrap();
            ptr += PAGE_SIZE;
        }
        Ok(())
    }

    /// Map a range of physical frame
    fn map_range(&mut self, r: FrameRange, address: usize) -> Result<()> {
        if !is_page_aligned!(address) {
            return Err(EFAULT);
        }
        // TODO assert aligned to page
        // TODO more generic api
        for i in 0..r.size {
            self.map_single(Frame(r.start.0 + i), address + i * PAGE_SIZE)
                .unwrap();
        }
        Ok(())
    }

    /// Will use the last entry of the page directory for recursive mapping
    fn virt_to_phys(&self, address: usize) -> Option<usize> {
        dbg!("Virt_to_phys input {:x}", address);
        // Index in PD
        let pde_index = pde_index!(address);
        let pte_offset = pte_index!(address);
        let offset = offset!(address);
        // Check if this entry is mapped, else we stop
        
        let pde = self.entries[pde_index];
        if pde.0 == 0 {
            return None;
        }

        dbg!("PDE = {:x}", pde.0);

        // TODO refactor
        if pde.0 & PDEF::PageSize.bits() != 0 {
            // NO PSE
            dbg!("PART 1< = {:x}", (pde.0 as usize & !((1 << 22) - 1)));
            let addr = (pde.0 as usize & !((1 << 22) - 1)) + (address & ((1 << 22) - 1));
            dbg!("Virt_to_phys 4MB page output {:x}", addr);
            return Some(addr);
        }


        let pt = unsafe { &*((self.entries[pde_index].0 as usize + super::KERNEL_LINEAR_START) as *const PageTable)};
        let addr = pt.entries[pte_offset] as usize + offset;
        dbg!("Virt_to_phys output {:x}", addr);
        Some(addr)
    }

    fn phys_to_virt(&self, address: usize) -> Option<usize> {
        if address > (super::KERNEL_TEMP_START - super::KERNEL_LINEAR_START) {
            return None
        }
        Some(KERNEL_LINEAR_START + address)
    }
}

/// Number of entries of 4MB to identity map in  the beginning
const NPDE_EARLY: usize = 10;

/// WARNING: this code will execute in protected mode, without paging enabled, before kstart
/// This means that any reference to symbols must be offsetted by the kernel virtual load address
/// we're effectively using real addresses
/// This function will linearly mapp the first 75% of physical memory
#[no_mangle]
pub extern "C" fn setup_early_paging() {
    // TODO check for PSE

    // For some reason, this actually resolves to the physical address and I have now idea why
    // It probably is calculated relatively, and I feel icky about it
    let pd_ptr = unsafe {
        &mut *((core::ptr::addr_of_mut!(KERNEL_PD) as usize - super::KERNEL_LINEAR_START)
            as *mut PageDir)
    };

    // I would expect something like that to be right
    // let pd_ptr: &mut PageDir = unsafe {&mut *(
    //     ( addr1 - super::KERNEL_LINEAR_START) as *mut PageDir
    // )};

    // Setting up the 1 to 1 identity mapping for the first few megabytes
    // We need this because we will return to a lower half physical address
    // This will get cleared up post higher half jump
    for i in 0..NPDE_EARLY {
        let phys_addr = (MB!(4) * i) as u32;
        pd_ptr.entries[i] = PDE::new(phys_addr, PDEF::Present | PDEF::PageSize | PDEF::Write);
    }
    // Setting up the linear mapping
    // i starts at the first kernel virtual address
    // we divied by 4mb to get the corresponding
    let start = super::KERNEL_LINEAR_START >> 22;
    let n = (super::KERNEL_TEMP_START >> 22) - start;
    for i in start..start + n {
        let phys_addr = (MB!(4) * (i - start)) as u32;
        pd_ptr.entries[i] = PDE::new(
            phys_addr,
            PDEF::Present | PDEF::PageSize | PDEF::Write | PDEF::Global,
        );
    }
    activate_paging();
}

/// Activate paging and PSE
#[no_mangle]
pub extern "C" fn activate_paging() {
    unsafe {
        // activate pse, move KERNEL_PD to cr3 and activate paging bit
        asm!(
            concat!(
                "
            mov eax, cr4
            or eax, ecx
            mov cr4, eax

            mov eax, edx
            mov cr3, eax

            mov eax, cr0
            or eax, 0x80000000
            mov cr0, eax"
            ),
            in("ecx") 1 << 4,
            in("edx") core::ptr::addr_of!(KERNEL_PD) as usize - super::KERNEL_LINEAR_START,
        );
    }
}

/// Remove the early identity mapping since we're in higher half now
pub fn cleanup_post_jump() {
    unsafe {
        for i in 0..NPDE_EARLY {
            KERNEL_PD.entries[i] = PDE::new(0, PDEF::empty());
        }
        flush_tlb();
    }
}

// TODO is this code archiecture specific ?
pub fn page_fault_handler(_instruction_pointer: u32, code: u32) {
    let address: u32;
    klog!("PAGE FAULT EXCEPTION");
    unsafe {
        asm!("mov {0}, cr2", out(reg) address);
    }
    klog!("Virtual address : {:p}", (address as *const u32));
    kprint!("Error code: "); // TODO reformat in the future
    let flags = PF::from_bits(code).unwrap();
    kprint!(
        "{} ",
        if flags.contains(PF::P) {
            "PAGE_PROTECTION"
        } else {
            "PAGE_NOT_PRESENT"
        }
    );
    kprint!(
        "{} ",
        if flags.contains(PF::W) {
            "WRITE"
        } else {
            "READ"
        }
    );
    if flags.contains(PF::U) {
        kprint!("CPL_3 ")
    };
    if flags.contains(PF::R) {
        kprint!("RESERVED_WRITE_BITS ")
    };
    if flags.contains(PF::I) {
        kprint!("INSTRUCTION_FETCH ")
    };
    if flags.contains(PF::PK) {
        kprint!("KEY_PROTECTION ")
    };
    if flags.contains(PF::SS) {
        kprint!("SHADOW STACK ")
    };
    if flags.contains(PF::SGX) {
        kprint!("SGX_VIOLATION ")
    };
    kprint!("\n");
    loop {}
}
