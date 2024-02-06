use crate::arch::paging;
use super::{PageManager, Frame};


// TODO this is a naive page manager, only for bootstraping the kernel development
// TODO locking
pub const BITMAP_SIZE : usize = paging::N_PAGES/8; // enough to manage 4GB

pub struct BitMap
{
  pub data : [u8; BITMAP_SIZE]
}

impl BitMap
{
  pub fn new() -> Self
  {
    BitMap{data: [0; BITMAP_SIZE]}
  }
}

impl PageManager for BitMap
{
  fn alloc_page(&mut self) -> Option<Frame>
  {
    for i in 0..BITMAP_SIZE {
        if self.data[i / 8] & (1 << (i % 8)) != 0 { // TODO could I speed up this module with & ?
          self.data[i / 8] |= 1 << (i % 8);
          return Some(Frame(i));
        }
    }
    None
  }

  fn free_page(&mut self, f: Frame)
  {
      self.data[f.0 / 8] &= !(1 << (f.0 % 8)); // TODO not sustainable if Frame changes
  }
}


