use alloc::vec::Vec;

pub trait BlockDriver {
    fn read(&self, lba: usize, buffer: &mut [u8]) -> Result<(), ()>;
    // fn write_block(&self, lba: usize, buffer: &[u8]);
    fn sector_size(&self) -> usize;
}

// TODO RW lock
static mut BLOCKS_DEVS: Vec<Rc<RefCell<dyn BlockDriver>>> = vec![];
pub fn get_devices() -> &'static Vec<Rc<RefCell<dyn BlockDriver>>> {
    unsafe { &BLOCKS_DEVS }
}

// enum DiskScheme {
//     MBR,
//     GPT
// }

// pub fn inspect_scheme(disk: &dyn BlockDriver) {
//     disk.read_block(0, buffer)
// }

// pub fn collect_partitions() {
// }

use alloc::rc::Rc;
use core::cell::RefCell;

use super::ext2;
use super::fs::FilesystemInit;

pub fn register_device(dev: Rc<RefCell<dyn BlockDriver>>) {
    unsafe {
        BLOCKS_DEVS.push(dev);
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
        for dev in BLOCKS_DEVS.iter_mut() {
            {
                let driver = dev.borrow_mut();
                driver.read(0, &mut buffer).unwrap();
            }

            // Read first block of device using lba
            // MBR partition
            if buffer[510] == 0x55 && buffer[511] == 0xAA {
                let mbr: &MBR = unsafe { &*(buffer.as_ptr() as *const MBR) };
                let mut superblock = [0 as u8; 512];
                let dest = mbr.parts[1].lba_start as usize;

                {
                    let driver = dev.borrow_mut();
                    driver.read(dest + 2, &mut superblock).unwrap();
                }

                if ext2::Ext2::match_superblock(&superblock) {
                    ext2::Ext2::init(dest, dest + 2, &dev).unwrap();
                }
            }
        }
    }
}
