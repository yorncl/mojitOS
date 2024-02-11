use crate::memory::Frame;

pub trait Mapper
{
    fn map_to_virt(f: Frame, address: usize) -> Result<(), ()>;
    fn virt_to_phys(address: usize) -> usize;
}
