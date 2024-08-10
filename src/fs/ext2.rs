use super::block::BlockDriver;
use super::vfs::{self, Filesystem};
use crate::klog;
use alloc::sync::Arc;
use core::cell::RefCell;

#[repr(C, packed)]
#[derive(Copy, Clone)]
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

pub struct Ext2 {
    info: vfs::Info,
    sb: SuperBlock,
    driver: Arc<dyn BlockDriver>,
}

impl Ext2 {
    pub fn get_block_group(&self, index: usize) {
        klog!("total : {}", { self.sb.total_blocks });
        klog!("per group : {}", { self.sb.blocks_per_group });
        klog!(
            "n block groups : {}",
            self.sb.total_blocks / self.sb.blocks_per_group
        );
    }
}

impl Filesystem for Ext2 {
    fn get_root(&self) -> Result<vfs::Inode, ()> {
        let rootindex = 2;
        let bgi = (rootindex - 1) / self.sb.inodes_per_group as usize;

        let mut buffer = [0 as u8; 512];

        self.get_block_group(bgi);
        // self.driver.borrow().read(0, );
        // self.driver.borrow().read();
        Err(())
    }
}

impl vfs::FilesystemInit for Ext2 {
    fn match_superblock(buffer: &[u8]) -> bool {
        let sb = unsafe { &*(buffer.as_ptr() as *const SuperBlock) };
        sb.ext2_signature == 0xef53
    }

    fn init(
        abs_lba_start: usize,
        abs_lba_super: usize,
        driver: Arc<dyn BlockDriver>,
    ) -> Result<Arc<Ext2>, ()> {
        let mut buffer = [0 as u8; 512];
        // Read the superblock
        //

        // TODO ugly, in braces for the ref to drop
        {
            let drv = driver.clone();
            // let handler = drv.borrow_mut();
            drv.read(abs_lba_super, &mut buffer).unwrap();
        }

        let sb = unsafe { &*(buffer.as_ptr() as *const SuperBlock) };

        // Checks for ext2 version >= 1, expecting superblock extensions
        // TODO handle exten
        if { sb.major_version } < 1 {
            return Err(());
        }

        let fs = Ext2 {
            info: vfs::Info {
                lba_offset: { sb.superblock_index } as usize,
                block_size: 1024 << { sb.block_size_log2 },
            },
            sb: sb.clone(),
            driver: driver.clone(),
        };
        Ok(Arc::new(fs))
    }
}
