use super::block::BlockDriver;

#[repr(C, packed)]
pub struct SuperBlock {
    total_inodes: u32,
    total_blocks: u32,
    superuser_blocks: u32,
    free_blocks: u32,
    free_inodes: u32,
    superblock_index: u32,
    block_size_log2: u32,
    frag_size_log2: u32,
    blocks_per_group: u32,
    frag_per_group: u32,
    inodes_per_group: u32,
    // POSIX time
    last_mount: u32,
    // POSIX time
    last_written: u32,
    fsck_count: u16,
    fsck_limit: u16,
    ext2_signature: u16,
    state: u16,
    error: u16,
    minor_version: u16,
    fsck_last_time: u32,
    fsck_interval: u32,
    osid: u32,
    major_version: u32,
    uid_reserved: u16,
    guid_reserved: u16,
}

// Compile time checks to ensure correct size of structures
const _: [u8; 84] = [0 as u8; core::mem::size_of::<SuperBlock>()];

use super::fs;

pub struct Ext2 {
    info: fs::Info,
    driver: Rc<RefCell<dyn BlockDriver>>,
}

use alloc::boxed::Box;
use alloc::rc::Rc;
use core::cell::RefCell;

impl Ext2 {

//     fn open_inode(&self, lba: usize, buffer: &[u8]) -> {
//         // TODO fix those borrow_mut shenanigans
//         let drv = self.driver.borrow_mut();

//         // drv.read_block(lba, &mut buffer);
//     }
}

impl fs::Filesystem for Ext2 {
}

impl fs::FilesystemInit for Ext2 {
    fn match_superblock(buffer: &[u8]) -> bool {
        let sb = unsafe { &*(buffer.as_ptr() as *const SuperBlock) };
        sb.ext2_signature == 0xef53
    }

    fn init(
        abs_lba_start: usize,
        abs_lba_super: usize,
        driver: &Rc<RefCell<dyn BlockDriver>>,
    ) -> Result<RefCell<Box<Ext2>>, ()> {
        let mut buffer = [0 as u8; 512];
        // Read the superblock
        //

        // TODO ugly, in braces for the ref to drop 
        {
            let drv = driver.clone();
            let handler = drv.borrow_mut();
            handler.read(abs_lba_super, &mut buffer).unwrap();
        }

        let sb = unsafe { &*(buffer.as_ptr() as *const SuperBlock) };

        // Checks for ext2 version >= 1, expecting superblock extensions
        // TODO handle exten
        if {sb.major_version} < 1 {
            return Err(())
        }

        let fs = Box::new(Ext2 {
            info: fs::Info {
                lba_offset: {sb.superblock_index} as usize,
                block_size: 1024 << {sb.block_size_log2},
            },
            driver: driver.clone(),
        });


        crate::klog!("{}", abs_lba_start);
        crate::klog!("{}", abs_lba_super);
        crate::klog!("{}", {sb.block_size_log2});
        crate::klog!("{}", {sb.total_inodes});
        crate::klog!("{}", {sb.free_inodes});

        Ok(RefCell::new(fs))
    }
}
