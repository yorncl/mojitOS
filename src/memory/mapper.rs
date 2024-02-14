use crate::memory::Frame;

pub use crate::arch::paging::Mapper;

pub trait MapperInterface
{
    fn map_to_virt(f: Frame, address: usize) -> Result<(), ()>;
    fn virt_to_phys(address: usize) -> Option<usize>;
    fn flush();
}
