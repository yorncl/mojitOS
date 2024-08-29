use super::PAGE_SIZE;
use crate::klib::mem::memset;
use crate::memory::pmm;
use crate::memory::pmm::{Frame, FrameRange};
use crate::memory::vmm::mapper;
use crate::utils::rawbox::RawBox;
use crate::x86::KERNEL_PAGE_TABLES_START;
use crate::{dbg, klog, kprint};
use bitflags::bitflags;
use core::arch::asm;
use core::ops::DerefMut;
use core::ffi::c_void;

pub static mut MAPPER: RawBox<PageDir> = RawBox {
    data: 0 as *mut PageDir,
};

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

extern "C" {
    static EPD_PHYS: PageDir;
    static EARLY_PAGE_DIRECTORY: PageDir;
}

pub(crate) use ROUND_PAGE_UP;

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
pub struct PageDir {
    entries: [PDE; 1024],
}

impl PageDir {
    #[inline(always)]
    pub fn set_entry(&mut self, i: usize, address: usize, flags: usize) {
        self.entries[i] = (address & !0xfff) | flags; // TODO convert to flags
    }

    pub fn dump_dbg(&self) {
        dbg!("Dumping page directory");
        for i in 0..1024 {
            if self.entries[i] != 0 {
                dbg!(" pd entry {} -> {:x} ({:x} - {:x})", i, self.entries[i], ((i as usize) << 22), ((i + 1 as usize) << 22));
                let special = (0x3ff << 22) | i << 12;
                dbg!("  Page table");
                let pt: &mut PageTable = unsafe { &mut *(special as *mut PageTable) };

                let mut count = -1;
                let mut prev: usize = {pt.entries[0]};
                let mut pdelta: i64 = 0;
                // continue;

                for j in 0..1024 {
                    let entry = {pt.entries[j]};

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
                        dbg!("   pt entry {} -> {:x} ({:x})", j, entry, ((i as usize) << 22 | (j as usize) << 12));
                    }
                    prev = entry;
                }
            }
        }
    }
}

impl Default for PageDir {
    fn default() -> Self {
        PageDir { entries: [0; 1024] }
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
    unsafe { MAPPER.deref_mut() }
}

// TODO the most abhorrent code i've written in my life so far
fn get_kernel_pt(index: usize) -> *mut PageTable {
    let ptr: *mut PageTable;
    ptr = (KERNEL_PAGE_TABLES_START + index * core::mem::size_of::<PageTable>()) as *mut PageTable;
    // TODO move out of the way
    ptr
}

impl mapper::MapperInterface for PageDir {
    /// Map a single physical frame to a virtual address
    fn map_single(&mut self, f: Frame, address: usize) -> Result<(), ()> {
        dbg!("Mapping page to {:x} ", address);
        if let Some(mapped) = self.virt_to_phys(address) {
            dbg!(
                "Mapping already mapped address {:x}, currently mapped to {:x} ",
                address,
                mapped
            );
            panic!("Mapping already mapped page");
        }
        if !is_page_aligned!(address) {
            return Err(());
        }
        let phys_address = f.0 * PAGE_SIZE;
        let pde_index = pde_index!(address);
        let pt = unsafe { &mut (*get_kernel_pt(pde_index)) };
        if self.entries[pde_index] == 0 {
            self.entries[pde_index] =
                self.virt_to_phys(pt as *const PageTable as usize).unwrap() | 3;
            memset(pt as *const PageTable as *mut c_void, 0, 4096);
        }
        pt.entries[pte_index!(address)] = phys_address | 3;

        // self.dump_dbg();
        flush_tlb();
        Ok(())
    }

    /// Map a single page and release its physical frame
    fn unmap_single(&mut self, address: usize) -> Result<(), ()> {
        if !is_page_aligned!(address) {
            return Err(());
        }
        dbg!("Unmapping a single frame");
        // Making sure that the page is mapped
        let address = self.virt_to_phys(address).ok_or(())?;

        // free the entry in the page directory
        // Use recursive mapping to get to the corresponding page table
        // The offset is to 0 so we have the start of the table
        let special = (0x3ff << 22) | pde_index!(address) << 12;
        let pt: &mut PageTable = unsafe { &mut *(special as *mut PageTable) };

        todo!();

        // Release the physical frame
        pmm::free_page(Frame(address / PAGE_SIZE));
        flush_tlb();
        Ok(())
    }

    /// Unmap multiple pages and release their physical frames
    fn unmap_range(&mut self, address: usize, npages: usize) -> Result<(), ()> {
        let mut ptr = address;
        for _ in 0..npages {
            self.unmap_single(ptr).unwrap();
            ptr += PAGE_SIZE;
        }
        Ok(())
    }

    /// Map a range of physical frame
    fn map_range(&mut self, r: FrameRange, address: usize) -> Result<(), ()> {
        if !is_page_aligned!(address) {
            return Err(());
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
        // Index in PD
        let pde_index = pde_index!(address);
        // Check if this entry is mapped, else we stop
        if self.entries[pde_index] == 0 {
            return None;
        }
        let pte_offset = pte_index!(address) * core::mem::size_of::<PTE>();
        let offset = offset!(address);

        // Here we exploit our recursive mapping
        // 0x3ff - the last entry of the PD
        // pde_index << 12 - will act as the PT index
        // pte_offsett - will be the offset to the entry that matter in the accessed Page Table
        let special = (0x3ff << 22) | pde_index << 12 | pte_offset;
        let pte: usize;
        unsafe {
            // We access that location which corresponds to the PT entry , and discard the lowest 12 flag bits
            pte = *(special as *const usize) & !0xfff;
        }
        // if the entry is to 0, then it is not mapped
        if pte == 0 {
            return None;
        }
        Some(pte + offset)
    }
}

// TODO might put this in the assembly
static mut KERNEL_PT_TEMP: [usize; 1024] = [0; 1024];

pub fn init_post_jump() {
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
        // We are making sure that the virtual address is well aligned and within the last
        // index of the directory
        assert!(pde_index!(KERNEL_PAGE_TABLES_START) == 0x3fd);
        assert!(is_page_aligned!(KERNEL_PAGE_TABLES_START));
        let address = mapper::virt_to_phys_kernel(KERNEL_PT_TEMP.as_ptr() as *const usize as usize)
            .expect("Cannot map KERNEL_PT_TMP");
        dir.set_entry(pde_index!(KERNEL_PAGE_TABLES_START), address, 3);

        // flush the tlb so we can map in the new table
        flush_tlb();

        // allocating 4MB of pages to store kernel pages TODO might be a bit overkill and
        // unoptimized
        let range =
            pmm::alloc_contiguous_pages(1024).expect("Cannot allocate page tables memory space");

        // Manually map the range, as mapper::map_range_kernel requires the kernel allocator to
        // be initialized, which itself needs paging (what we are doing right now you dingus)
        let start = range.start.0;
        for i in 0..range.size {
            KERNEL_PT_TEMP[i] = start + i * PAGE_SIZE | 3; // TODO better flags
        }
        // flush the tlb one last time so that the new table is updated
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
