// use super::super::Address;


// Bump allocator, used to bootstrap the other allocators
pub struct Bump
{
    pub start: usize, // TOOD shoudl I use crate::memory::Address type ?
    pub size: usize
}

impl Bump
{
    fn new(start: usize, size: usize)
    {
        Bump {start, size};
    }

    pub fn allocate(&mut self, n: usize) -> usize
    {
        let offset = self.size;
        self.size += n;
        self.start + offset
        // TODO check for new pages
    }

}

