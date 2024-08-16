use core::fmt::Debug;
use core::marker::PhantomData;
use core::mem::size_of;
use crate::dbg;
use crate::error::{Result, EUNKNOWN};
use crate::fs::block::{Lba, Partition};
use crate::fs::vfs::{self, FileSystemSetup, Filesystem, Inonum};
use alloc::sync::Arc;

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
pub struct Inode {
    mode: u16,
    uid: u16,
    /// low 32 bit of size
    size_low: u32,
    /// Last of each POSIX timestamp
    access: u32,
    creation: u32,
    modification: u32,
    delete: u32,
    guid: u16,
    hard_links: u16,
    // TODO I'll have to clean the API to handle that
    disk_sec: u32,
    flags: u32,
    osid: u32,
    block_ptr: [u32; 12],
    sinlgly_ptr: u32,
    doubly_ptr: u32,
    triply_ptr: u32,
    gen_number: u32,
    ext_attr: u32,
    size_upper: u32,
    frag: u32,
    os_frag: [u8; 12],
}

#[repr(C, packed)]
#[derive(Copy, Clone)]
pub struct BGDescriptor {
    block_bitmap: u32,
    inode_bitmap: u32,
    inode_table: u32,
    free_blocks: u16,
    free_inodes: u16,
    n_directories: u16,
    _unused: [u8; 14],
}

#[repr(C, packed)]
#[derive(Copy, Clone)]
pub struct Dentry {
    inode: u32,
    size: u16,
    lb_name_length: u8,
    type_indicator: u8,
    _padd: [u8; 4],
}

// Compile time checks to ensure correct size of structures
const _: [u8; 84] = [0 as u8; core::mem::size_of::<SuperBlock>()];
const _: [u8; 32] = [0 as u8; core::mem::size_of::<BGDescriptor>()];
const _: [u8; 128] = [0 as u8; core::mem::size_of::<Inode>()];
const _: [u8; 12] = [0 as u8; core::mem::size_of::<Dentry>()];

impl Dentry {
    pub fn name(&self) -> Option<&'static str> {
        let name_start = core::ptr::addr_of!(self._padd) as *const u8; // \0 character
        Some(unsafe {
            core::str::from_utf8(core::slice::from_raw_parts(
                name_start,
                self.lb_name_length as usize,
            ))
            .unwrap()
        })
    }
}

/// Wrapper around a raw buffer containing directory entries
/// Implements the iterator trait for ergonnomics
pub struct DirBlock<'a> {
    ptr: *const u8,
    boundary: usize,
    _marker: PhantomData<&'a ()>,
}

impl<'a> DirBlock<'a> {
    pub fn new(ptr: *const u8, size: usize) -> DirBlock<'a> {
        DirBlock {
            ptr,
            boundary: (ptr as usize) + size,
            _marker: PhantomData,
        }
    }
}

impl<'a> Iterator for DirBlock<'a> {
    type Item = &'a Dentry;

    fn next(&mut self) -> Option<Self::Item> {
        if (self.ptr as usize) >= self.boundary {
            return None;
        }
        let entry = unsafe { &*(self.ptr as *const Dentry) };
        if entry.inode == 0 {
            return None;
        }
        let mut total_size = size_of::<Dentry>() as isize
            + core::cmp::max(entry.lb_name_length as isize - 4, 0) as isize;

        // Round up to the next u32 boundary
        total_size = (total_size + 3) & !3;
        dbg!("Total size to jump {}", total_size);

        self.ptr = unsafe { self.ptr.offset(total_size) };
        return Some(entry);
    }
}

impl Debug for Dentry {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(
            f,
            "Dentry  inode {}, size {}, lb_name_length {}, name \"{}\"",
            { self.inode },
            { self.size },
            { self.lb_name_length },
            // TODO will get funny if a dir is actually named that way
            self.name().unwrap_or("CANNOT READ DIR NAME")
        )
    }
}

/// The Ext2 fileystem interface
pub struct Ext2 {
    info: vfs::Info,
    sb: SuperBlock,
    part: Arc<Partition>,
}

impl Ext2 {
    fn get_inode(&self, inode_num: Inonum) -> Result<Lba> {
        // Figure out in which block group the inode is
        let block_group = ((inode_num as u32 - 1) / self.sb.inodes_per_group) as usize;
        let bgd = self.get_bg_descriptor(block_group)?;
        // dbg!(
        //     "block {} inode {}, inode table {} dirs {}",
        //     { bgd.block_bitmap },
        //     { bgd.inode_bitmap },
        //     { bgd.inode_table },
        //     { bgd.n_directories }
        // );

        // TODO clear up type and casts, remove fs/drive constants and place them in structs
        // TODO clean up the casts, less of it please
        // getting the block that contains the right portion of the inode table
        let mut buffer = [0 as u8; 1024];
        let mut inode_table_i = (inode_num as usize - 1) % self.sb.inodes_per_group as usize;
        let inode_per_block = 1024 / size_of::<Inode>();
        let table_block_offset = (inode_table_i as usize as usize * size_of::<Inode>()) / 1024;
        inode_table_i -= inode_per_block * table_block_offset;

        self.part.read(
            (bgd.inode_table as usize + table_block_offset) as Lba,
            &mut buffer,
        )?;
        let inode = unsafe { *(buffer.as_ptr() as *const Inode).offset(inode_table_i as isize) };

        self.part.read(inode.block_ptr[0] as Lba, &mut buffer)?;
        let db = DirBlock::new(buffer.as_ptr() as *const u8, 1024);
        // dbg!("========= Start entries");
        // for _dir in db.into_iter() {
        //     dbg!("{:?}", _dir);
        // }
        // dbg!("Reached the end do'");
        Ok(0)
    }

    fn get_bg_descriptor(&self, index: usize) -> Result<BGDescriptor> {
        let mut buffer = [0 as u8; 1024];

        dbg!("INODES PER GROUP {}", { self.sb.inodes_per_group });

        // let block_addr = self.sb.superblock_index as Lba
        //     + ((index * size_of::<BGDescriptor>()) / self.info.block_size) as Lba
        //     + 1;
        let block_addr = 2;
        let bgd_offset = ((index * size_of::<BGDescriptor>()) % self.info.block_size) as usize;
        dbg!(
            "index {} Block addr {}, offset {}",
            index,
            block_addr,
            bgd_offset
        );
        self.part.read(block_addr, &mut buffer)?;
        // unsafe {
        //     let mut pointer = (buffer.as_ptr() as *const BGDescriptor);
        //     for i in 0..1024/size_of::<BGDescriptor>() {
        //         let bgd = *pointer;
        //         dbg!(
        //             "block {} inode {}, inode table {} dirs {}",
        //             {bgd.block_bitmap},
        //             {bgd.inode_bitmap},
        //             {bgd.inode_table},
        //             {bgd.n_directories}
        //         );
        //         pointer = pointer.offset(1);
        //     }
        // }
        Ok(unsafe { *(buffer.as_ptr().offset(bgd_offset as isize) as *const BGDescriptor) })
    }
}

impl FileSystemSetup for Ext2 {
    fn try_init(part: Arc<Partition>) -> Result<Option<Arc<Ext2>>> {
        let mut buffer = [0 as u8; 1024];

        // Read the superblock
        // TODO I'm not clear on the buffer size still
        part.read(1, &mut buffer)?;
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
        // TODO this will be offsetted by -1, maybe not clear enough
        let rootindex: Inonum = 3;
        Ok(rootindex)
    }

    fn read_inode(&self, inode: Inonum) -> Result<vfs::Vnode> {
        // self.driver.read
        self.get_inode(inode);
        Err(EUNKNOWN)
    }

    fn read(&self) -> Result<usize> {
        unimplemented!()
    }
}
