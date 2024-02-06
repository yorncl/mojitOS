mod bitmap;

use bitmap::{BitMap, BITMAP_SIZE};

pub static mut PMM : BitMap = BitMap {data: [0; BITMAP_SIZE]};

// Abstract representation of a frame
use crate::arch::paging::Frame;

// Physical Memory Manager trait
// Every physical memory manager should implement this trait
pub trait PageManager
{
  fn alloc_page(&mut self) -> Option<Frame>;
  // fn alloc_pages(n: usize); TODO multiple frame at once like a boos
  fn free_page(&mut self, f: Frame);
}
