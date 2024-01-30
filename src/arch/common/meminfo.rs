// Holds the information about the physical memory map

struct MemInfo {
  main_start : usize,
  main_size : usize
}

// TODO conditional variable based on architecture
pub const PAGE_SIZE : usize = 0x1000;
pub const MIN_REQUIRED_MEMORY : usize = 0x10000; // 10 MB 

static mut MEM_INFO : MemInfo = MemInfo{main_start : 0, main_size: 0};

pub fn get_mem_start() -> usize
{
  unsafe { MEM_INFO.main_start }
}

pub fn get_mem_size() -> usize
{
  unsafe { MEM_INFO.main_size }
}

pub fn set_mem_info(start: usize, size: usize)
{
  unsafe { MEM_INFO.main_start = start };
  unsafe { MEM_INFO.main_size = size };
}
