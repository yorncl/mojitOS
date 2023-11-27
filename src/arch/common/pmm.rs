use crate::klog;

const PMM_SIZE : usize = 1 << 17; // TODO enough to manage 4GB
static mut PMM : [u8; PMM_SIZE] = [0; PMM_SIZE]; // TODO probably shouldn't be static allocation ?

macro_rules! is_set
{
  ($a:expr) => {
    if PMM[$a / 8] & (1 << ($a % 8)) != 0 {
      true
    }
    else {
      false
    }
  }
}
macro_rules! set_bit
{
  ($a:expr) => {
    PMM[$a / 8] |= (1 << ($a % 8));
  }
}
macro_rules! unset_bit
{
  ($a:expr) => {
    PMM[$a / 8] &= !(1 << ($a % 8));
  }
}

// get first free page and returns its address
fn get_first_free() -> Option<usize> // TODO refactor to Result ?
{
  for i in 0..PMM_SIZE {
    unsafe {
      if !is_set!(i) {
        return Some(i);
      }
    }
  }
  None
}

fn alloc_block(i: usize)
{
  unsafe {
    set_bit!(i);
  }
}

fn free_block(i: usize)
{
  unsafe {
    unset_bit!(i);
  }
}

pub fn init()
{
  unsafe {
    match get_first_free() 
    {
      Some(block) => { alloc_block(block); klog!("Allocating block number {}, PMM[0]={})", block, PMM[0]); },
      None => klog!("Oh shoot we're out of memory !!!!!")
    }
  }
}
