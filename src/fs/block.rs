

pub trait BlockDriver {
    fn read_block(&self, lba: usize);
}
