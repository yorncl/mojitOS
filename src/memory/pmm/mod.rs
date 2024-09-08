mod bitmap;

use crate::error::Result;
use crate::klib::lock::RwLock;
use crate::memory::{PhysicalMemory, RegionType, PAGE_SIZE};
use bitmap::BitMap;
use core::fmt;

/// Abstract representation of Frame
/// It does not represent an address because the PMM shouldn't be aware of the current
/// architecture, beyond that it is a page based system
pub struct Frame(pub usize);

impl fmt::Display for Frame {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "({}, {})", self.0, self.0)
    }
}

pub struct FrameRange {
    pub start: Frame,
    pub size: usize,
}

/// The PMM instance
static mut PMM: RwLock<BitMap> = RwLock::new(BitMap::default_const());

/// Zone of the allocation
pub enum Zone {
    Normal,
    Dma,
}

/// Physical Memory Manager trait
/// Every physical memory manager should implement this trait
pub trait PageManager {
    fn setup(&mut self) -> Result<()>;
    fn alloc_page(&mut self, z: Zone) -> Result<Frame>;
    fn alloc_contiguous_pages(&mut self, n: usize, z: Zone) -> Result<FrameRange>;
    fn free_page(&mut self, f: Frame);
    fn free_contiguous_pages(&mut self, r: FrameRange);
    fn fill_range(&mut self, r: FrameRange) -> ();
    fn get_phys_frames(&self, phys_addres: usize, n: usize) -> FrameRange;
}

/// Initialize the physical memory manager
pub fn init(memmap: &PhysicalMemory) {
    for entry in memmap.regions {
        if entry.rtype == RegionType::Available {
            free_contiguous_pages(FrameRange {
                start: Frame(entry.start / PAGE_SIZE),
                size: entry.size / PAGE_SIZE,
            })
        }
    }
}

#[inline(always)]
/// Allocate a single physical page in a physical zone
pub fn alloc_page(zone: Zone) -> Result<Frame> {
    let mut pmm = unsafe { PMM.write().unwrap() };
    pmm.alloc_page(zone)
}

#[inline(always)]
/// Allocate n contiguous pages in a physical zone
pub fn alloc_contiguous_pages(n: usize, zone: Zone) -> Result<FrameRange> {
    let mut pmm = unsafe { PMM.write().unwrap() };
    pmm.alloc_contiguous_pages(n, zone)
}

#[inline(always)]
/// Free a single page
pub fn free_page(f: Frame) {
    let mut pmm = unsafe { PMM.write().unwrap() };
    pmm.free_page(f);
}

#[inline(always)]
/// Free a range of contiguous pages
pub fn free_contiguous_pages(f: FrameRange) {
    let mut pmm = unsafe { PMM.write().unwrap() };
    pmm.free_contiguous_pages(f)
}

#[inline(always)]
/// Marks the page ranges as allocated
/// among other things, used to block out reserved page ranges
pub fn fill_range(f: FrameRange) -> () {
    let mut pmm = unsafe { PMM.write().unwrap() };
    pmm.fill_range(f)
}

#[inline(always)]
pub fn get_phys_frames(phys_addres: usize, n: usize) -> FrameRange {
    let mut pmm = unsafe { PMM.write().unwrap() };
    pmm.get_phys_frames(phys_addres, n)
}
