use crate::memory::vmm::mapper;

// TODO remove altogether
pub fn remap_phys(phys_addr: usize, size: usize) -> Result<usize, &'static str> {
    mapper::phys_to_virt(phys_addr).ok_or("Could not remap physical address")
}
