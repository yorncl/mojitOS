use super::PageManager;
use crate::arch;
use crate::memory::{Frame, PhysicalRegion, RegionType};

// TODO this is a naive page manager, only for bootstraping the kernel development
// TODO locking
pub const BITMAP_SIZE: usize = arch::N_PAGES / 8; // enough to manage 4GB

pub struct BitMap {
    pub data: [u8; BITMAP_SIZE],
    pub start: usize,
    pub size: usize
}

// TODO add max address for physical memory

impl BitMap {
    pub fn new() -> Self {
        BitMap {
            data: [0; BITMAP_SIZE],
            start: 0,
            size: 0
        }
    }
}

impl PageManager for BitMap {
    fn alloc_page(&mut self) -> Option<Frame> { // TODO limit and out of memory error
        for i in 0..BITMAP_SIZE {
            if self.data[i / 8] & (1 << (i % 8)) == 0 {
                // TODO could I speed up this module with & ?
                self.data[i / 8] |= 1 << (i % 8);
                return Some(Frame(i));
            }
        }
        None
    }

    fn free_page(&mut self, f: Frame) {
        self.data[f.0 / 8] &= !(1 << (f.0 % 8)); // TODO not sustainable if Frame changes
    }

    fn fill_range(&mut self, start: usize, n: usize) {
        for i in 0..n { // TODO optimize for faster filling
            let index = start + i;
            self.data[index / 8] |= 1 << (index % 8);
        }
    }
}
