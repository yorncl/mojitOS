mod bitmap;

use crate::memory;
use bitmap::BitMap;

use self::bitmap::BITMAP_SIZE;

pub static mut PMM: BitMap = BitMap {
    data: [0; BITMAP_SIZE],
    start: 0,
    size: 0
};
// TODO proper non contiguous area managemetn

/// Physical Memory Manager trait
/// Every physical memory manager should implement this trait
pub trait PageManager {
    fn alloc_page(&mut self) -> Option<memory::Frame>;
    // fn alloc_pages(n: usize); TODO multiple frame at once like a boos
    fn free_page(&mut self, f: memory::Frame);

    fn fill_range(&mut self, start:usize, n: usize) -> ();
}
