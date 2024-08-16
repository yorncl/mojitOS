use crate::error::{Result, EUNKNOWN};
use crate::fs::{ext2, vfs, vfs::FileSystemSetup};
use crate::klib::lock::RwLock;
use crate::klog;
use alloc::sync::Arc;
use alloc::vec::Vec;

/// Logical Block Address
pub type Lba = u64;

pub trait BlockDriver {
    fn read(&self, lba: usize, buffer: &mut [u8]) -> Result<usize>;
    // fn write_block(&self, lba: usize, buffer: &[u8]);
    fn sector_size(&self) -> usize;
}

/// Block devices are registered here
static BLOCKS_DEVS: RwLock<Vec<Arc<RwLock<dyn BlockDriver>>>> = RwLock::new(Vec::new());

/// get references to block devices vector
pub fn get_devices() -> &'static RwLock<Vec<Arc<RwLock<dyn BlockDriver>>>> {
    &BLOCKS_DEVS
}

pub fn register_device(dev: Arc<RwLock<dyn BlockDriver>>) {
    let mut v = BLOCKS_DEVS.write().unwrap();
    v.push(dev);
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

// TODO interface to identify by name
/// The interface between the filesystem and the block driver
/// This enable filesystem drivers to address blocks without thinking about the whole disk
/// Eventually it might get integrated into/replaced by a request-based system, who knows
pub struct Partition {
    block_start: Lba,
    seccount: u64,
    pub dev: Arc<RwLock<dyn BlockDriver>>,
}

static PARTITIONS: RwLock<Vec<Arc<Partition>>> = RwLock::new(Vec::new());

impl Partition {
    #[inline]
    pub fn read(&self, lba: Lba, buffer: &mut [u8]) -> Result<usize> {
        let driver = self.dev.write().unwrap();
        // TODO fix this mapping 
        let block_index = self.block_start + lba * 2;
        driver.read(block_index as usize, buffer)
    }
}

// Loop through all the disks and extract filesystems volumes
pub fn init_fs_from_devices() {
    let devices = BLOCKS_DEVS.read().unwrap();
    if devices.len() == 0 {
        panic!("No block devices detected, are you sure a drive is connected ?");
    }

    let mut buffer = [0 as u8; 512];
    for dev_lock in devices.iter() {
        // Reading the first sector, and release the lock
        let dev = dev_lock.write().unwrap();
        dev.read(0, &mut buffer).unwrap();
        drop(dev);

        // Read first block of device using lba
        // MBR partition
        if buffer[510] == 0x55 && buffer[511] == 0xAA {
            let mut partitions = PARTITIONS.write().unwrap();
            let mbr: &MBR = unsafe { &*(buffer.as_ptr() as *const MBR) };

            for i in 0..4 {
                let mbpart = &mbr.parts[i];
                let part = Arc::new(Partition {
                    block_start: mbpart.lba_start as Lba,
                    seccount: mbpart.seccount as u64,
                    dev: dev_lock.clone(),
                });
                partitions.push(part.clone());

                // check for ext2
                // superblocka at 1024 offset TODO more clear on block size while addressing
                match ext2::Ext2::try_init(part.clone()) {
                    Ok(val) => {
                        if val.is_some() {
                            vfs::get_filesystems().push(val.unwrap());
                            klog!("  Registered an ext2 filesystem");
                        }
                    }
                    Err(errcode) => {
                        klog!("Error while init ext2: {}", errcode);
                    },
                }
            }
        }
    }
}
