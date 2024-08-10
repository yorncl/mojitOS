use alloc::vec::Vec;
use crate::klib::lock::RwLock;

pub trait BlockDriver {
    fn read(&self, lba: usize, buffer: &mut [u8]) -> Result<(), ()>;
    // fn write_block(&self, lba: usize, buffer: &[u8]);
    fn sector_size(&self) -> usize;
}

static mut BLOCKS_DEVS: Vec<RwLock<Arc<dyn BlockDriver>>> = vec![];
pub fn get_devices() -> &'static Vec<RwLock<Arc<dyn BlockDriver>>> {
    unsafe { &BLOCKS_DEVS }
}

use alloc::sync::Arc;

use super::{ext2, vfs};
use super::vfs::FilesystemInit;

pub fn register_device(dev: Arc<dyn BlockDriver>) {
    unsafe {
        BLOCKS_DEVS.push(RwLock::new(dev));
    }
}

#[repr(C, packed)]
struct MBRPart {
    attributes: u8,
    // CHS start
    c_start: u8,
    h_start: u8,
    s_start: u8,
    part_type: u8,
    // CHS end
    c_last: u8,
    h_last: u8,
    s_last: u8,
    // LBA
    lba_start: u32,
    seccount: u32,
}

#[repr(C, packed)]
struct MBR {
    bin: [u8; 440],
    id: u32,
    _reserved: u16,
    parts: [MBRPart; 4],
    magic: u16,
}

impl MBR {}

// Loop through all the disks and extract filesystems volumes
pub fn init_fs_from_devices() {
    if get_devices().len() == 0 {
        panic!("No block devices detected, are you sure a drive is connected ?");
    }

    unsafe {
        let mut buffer = [0 as u8; 512];
        for dev_lock in BLOCKS_DEVS.iter_mut() {
            let dev = dev_lock.write().unwrap();
            dev.read(0, &mut buffer).unwrap();

            // Read first block of device using lba
            // MBR partition
            if buffer[510] == 0x55 && buffer[511] == 0xAA {
                let mbr: &MBR = &*(buffer.as_ptr() as *const MBR);
                let mut superblock = [0 as u8; 512];
                let dest = mbr.parts[1].lba_start as usize;

                dev.read(dest + 2, &mut superblock).unwrap();

                if ext2::Ext2::match_superblock(&superblock) {
                    match ext2::Ext2::init(dest, dest + 2, dev.clone()) {
                        Ok(val) => {
                            vfs::get_filesystems().push(val);
                        },
                        Err(()) => { continue }
                    }
                }
            }
        }
    }
}
