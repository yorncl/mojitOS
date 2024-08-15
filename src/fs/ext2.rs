use crate::dbg;
use crate::error::{Result, EUNKNOWN};
use crate::fs::block::{BlockDriver, Lba, Partition};
use crate::fs::vfs::{self, FileSystemSetup, Filesystem, Inonum};
use crate::klib::lock::RwLock;
use alloc::sync::Arc;

use super::block;

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

#[repr(C, packed)]
#[derive(Copy, Clone)]
pub struct BGDescriptor {
    block_bitmap: u32,
    inode_bitmap: u32,
    inode_table: u32,
    free_blocks: u16,
    free_inodes: u16,
    n_direcotries: u16,
    _unused: [u8; 14],
}

// Compile time checks to ensure correct size of structures
const _: [u8; 84] = [0 as u8; core::mem::size_of::<SuperBlock>()];
const _: [u8; 32] = [0 as u8; core::mem::size_of::<BGDescriptor>()];

/// The Ext2 fileystem interface
pub struct Ext2 {
    info: vfs::Info,
    sb: SuperBlock,
    part: Arc<Partition>,
}

impl Ext2 {
    pub fn get_inode_block(&self, inode: Inonum) -> Result<Lba> {
        let block_group = ((inode as u32 - 1) / self.sb.inodes_per_group) as usize;

        let bgd = self.get_bg_descriptor(block_group)?;
        todo!()
    }

    pub fn get_bg_descriptor(&self, index: usize) -> Result<BGDescriptor> {
        // let mut buffer = [0 as u8; 512];
        // let block_addr = (self.sb.block) as Lba;
        // self.part.read(block_addr: buffer)?;
        todo!()
    }
}

impl FileSystemSetup for Ext2 {
    fn try_init(part: Arc<Partition>) -> Result<Option<Arc<Ext2>>> {
        let mut buffer = [0 as u8; 512];

        // Read the superblock
        // TODO I'm not clear on the buffer size still
        part.read(2, &mut buffer)?;
        let sb = unsafe { &*(buffer.as_ptr() as *const SuperBlock) };

        if sb.ext2_signature != 0xef53 {
            return Ok(None);
        }

        // Checks for ext2 version >= 1, expecting superblock extensions
        // TODO handle exten
        if { sb.major_version } < 1 {
            return Ok(None);
        }

        let fs = Ext2 {
            info: vfs::Info {
                lba_offset: { sb.superblock_index } as usize,
                block_size: 1024 << { sb.block_size_log2 },
            },
            sb: sb.clone(),
            part: part.clone(),
        };
        Ok(Some(Arc::new(fs)))
    }
}

impl Filesystem for Ext2 {
    // TODO handle other ext2 versions
    fn get_root_inode(&self) -> Result<Inonum> {
        let rootindex: Inonum = 2;
        Ok(rootindex)
    }

    fn read_inode(&self, inode: Inonum) -> Result<vfs::Vnode> {
        // self.driver.read
        todo!()
    }

    fn read(&self) -> Result<usize> {
        unimplemented!()
    }
}
