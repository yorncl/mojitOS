use crate::memory::{pmm, PAGE_SIZE};
use crate::memory::vmm::mapper;
use crate::MB;
use crate::{ROUND_PAGE_UP, ROUND_PAGE_DOWN};

// TODO extremely primitive memory management

struct BumpMMIO {
    start: usize,
    size: usize
}

impl BumpMMIO {
    pub fn allocate(&mut self, nframes: usize) -> Result<usize, &'static str> {
        if self.size + nframes * PAGE_SIZE > MB!(1) {
            return Err("Out of MMIO memory");
        }
        let r = self.start + self.size;
        self.size += nframes * PAGE_SIZE;
        Ok(r)
    }
}

static mut IOMM: BumpMMIO = BumpMMIO{ 
    start: 0,
    size: 0,
};

pub fn init() -> Result<(), ()>{
    //TOOD early boot allocation to facilitate everything
    // Allocate a page for self storage
    // mapper::map_single_kernel(pmm::alloc_page().unwrap(), super::KERNEL_IOMM_START)?;
    todo!();
}

// Mapping a page to a physical address
// Mainly used for MMIO
pub fn remap_phys(phys_addr: usize, size: usize) -> Result<usize, &'static str> {
    // TODO better way to do this ?
    // here, we lose the bytes below the requested address due to alignment
    let phys_start = ROUND_PAGE_DOWN!(phys_addr);
    let phys_end = ROUND_PAGE_UP!(phys_addr + size);
    let nframes = (phys_end - phys_start) / PAGE_SIZE;

    unsafe {
        match IOMM.allocate(nframes) {
            Ok(vptr) => {
                let range = pmm::get_phys_frames(phys_start, nframes);
                // TODO Extremely bad error management
                // might have to redo the whole system more cleanly soon
                mapper::map_range_kernel(range, vptr).or(Err("Cannot map range kernel"));
                // adding the offset so that we get the address we needed for the structure
                let offset = phys_addr - phys_start;
                return Ok(vptr + offset);
            },
            Err(msg) => return Err(msg)
        }
    };
}
