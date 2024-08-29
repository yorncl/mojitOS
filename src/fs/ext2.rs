use crate::dbg;
use crate::error::{codes::*, Result};
use crate::fs::block::{BlockDev, Lba};
use crate::fs::vfs::{
    Dentry, Dirent, File, FileOps, FileSystemSetup, Filesystem, Info, Inonum, NodeOps, Vnode,
    VnodeType, NAME_MAX,
};
use alloc::sync::Arc;
use alloc::boxed::Box;
use core::fmt::Debug;
use core::mem::size_of;

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

#[derive(Copy, Clone)]
struct InoBlockBuff {
    // index of current cached buffer
    curr: Option<Lba>,
    // TODO concurrency issues with cached values, clean up
    data: [u32; NL0],
}

impl InoBlockBuff {
    #[inline]
    fn to_u8(&mut self) -> &mut [u8; NL0 * 4] {
        unsafe { &mut *(self.data.as_ptr() as *const u8 as *mut [u8; NL0 * 4]) }
    }

    /// Get an entry in buffer
    #[inline]
    fn get(&self, index: usize) -> Option<Lba> {
        let lba = self.data[index] as Lba;
        if lba == 0 {
            return None;
        }
        Some(lba)
    }
}

/// Interface to get blocks address in a linear fashion from a block index
struct InoBlocks {
    index: usize,
    // Cached buffers for each level
    // singly, doubly, triply
    buff: [InoBlockBuff; 3],
    driver: Arc<BlockDev>,
}

// Constants for number of accessible blocks through X dereference
// singly
const NL0: usize = 1024 / core::mem::size_of::<u32>();
// doubly
const NL1: usize = NL0 * NL0;
// triply
const NL2: usize = NL0 * NL0 * NL0;
// TODO test case for large files
impl InoBlocks {
    /// Takes a linear block address and returns the LBA if it exists
    fn get_lba(&mut self, inode: &Inode, block_index: usize) -> Result<Option<Lba>> {
        // let i = self.block_index as usize;
        // block_ptr fields are straighforward
        if block_index < 12 {
            return Ok(Some(inode.block_ptr[block_index] as Lba));
        }

        // offset the index
        let index = block_index - 12;
        let mut addr: Option<Lba> = None;
        // Match and check within which range it is
        match index {
            // singly
            0..NL0 => {
                addr = self.get_singly(inode, index)?;
            }
            // doubly
            NL0..NL1 => {
                addr = self.get_doubly(inode, index - NL0)?;
            }
            // triply
            NL1..NL2 => {
                addr = self.get_triply(inode, index - NL0 - NL1)?;
            }
            _ => {}
        }
        Ok(addr)
    }

    fn load_buff(&mut self, index: usize, lba: Lba) -> Result<()> {
        let curr = self.buff[index].curr;
        // load if not cached
        if curr.is_none() || curr.unwrap() != lba {
            self.driver.read(lba, self.buff[index].to_u8())?;
            // setting the last cached buffer
            self.buff[index].curr = Some(lba);
        }
        Ok(())
    }

    // Fetch the singly buffer and returns the Lba at index
    fn get_singly(&mut self, inode: &Inode, index: usize) -> Result<Option<Lba>> {
        if inode.sinlgly_ptr == 0 {
            return Ok(None);
        }
        // If not cached, fetch it
        self.load_buff(0, inode.sinlgly_ptr as Lba)?;

        // Get the lba at index
        Ok(self.buff[0].get(index))
    }

    // Fetch the doubly buffer and returns the Lba at index
    fn get_doubly(&mut self, inode: &Inode, index: usize) -> Result<Option<Lba>> {
        if inode.doubly_ptr == 0 {
            return Ok(None);
        }
        // Load first level
        self.load_buff(0, inode.doubly_ptr as Lba)?;
        if let Some(lba) = self.buff[0].get(index/NL0) {
            // Load second level
            self.load_buff(1, lba)?;
            return Ok(self.buff[1].get(index%NL0));
        }
        Ok(None)
    }

    // Fetch the triply buffer and returns the Lba at index
    fn get_triply(&mut self, inode: &Inode, index: usize) -> Result<Option<Lba>> {
        if inode.doubly_ptr == 0 {
            return Ok(None);
        }

        // Load first level
        self.load_buff(0, inode.doubly_ptr as Lba)?;
        if let Some(lba) = self.buff[0].get(index/NL1) {
            // Load second level
            self.load_buff(1, lba)?;
            if let Some(lba) = self.buff[1].get((index%NL1)/NL0) {
                // Load third level
                self.load_buff(2, lba)?;
                return Ok(self.buff[2].get(index%NL0));
            }
        }
        Ok(None)
    }
}

impl Inode {
    fn blocks(&'_ self, drv: Arc<BlockDev>) -> InoBlocks {
        InoBlocks {
            index: 0,
            buff: [InoBlockBuff {
                curr: None,
                data: [0; NL0],
            }; 3],
            driver: drv,
        }
    }
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
pub struct Ext2Dentry {
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
const _: [u8; 12] = [0 as u8; core::mem::size_of::<Ext2Dentry>()];

impl Ext2Dentry {
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

impl Debug for Ext2Dentry {
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
    info: Info,
    sb: SuperBlock,
    block_dev: Arc<BlockDev>,
}

impl Ext2 {
    fn get_inode(&self, inode_num: Inonum) -> Result<Inode> {
        // Figure out in which block group the inode is
        let block_group = ((inode_num as u32 - 1) / self.sb.inodes_per_group) as usize;
        let bgd = self.get_bg_descriptor(block_group)?;

        // TODO clear up type and casts, remove fs/drive constants and place them in structs
        // TODO clean up the casts, less of it please
        // getting the block that contains the right portion of the inode table
        let mut buffer = [0 as u8; 1024];
        let mut inode_table_i = (inode_num as usize - 1) % self.sb.inodes_per_group as usize;
        let inode_per_block = 1024 / size_of::<Inode>();
        let table_block_offset = (inode_table_i as usize as usize * size_of::<Inode>()) / 1024;
        inode_table_i -= inode_per_block * table_block_offset;

        self.block_dev.read(
            (bgd.inode_table as usize + table_block_offset) as Lba,
            &mut buffer,
        )?;
        let inode = unsafe { *(buffer.as_ptr() as *const Inode).offset(inode_table_i as isize) };

        // dbg!("========= Start entries");
        // for _dir in db.into_iter() {
        //     dbg!("{:?}", _dir);
        // }
        // dbg!("Reached the end do'");
        Ok(inode)
    }

    fn get_bg_descriptor(&self, index: usize) -> Result<BGDescriptor> {
        // TODO Figure out the buffer situation
        let mut buffer = [0 as u8; 1024];

        // TODO shouldn't be hardcorded
        let block_addr = 2;
        let bgd_offset = ((index * size_of::<BGDescriptor>()) % self.info.block_size) as usize;
        self.block_dev.read(block_addr, &mut buffer)?;
        Ok(unsafe { *(buffer.as_ptr().offset(bgd_offset as isize) as *const BGDescriptor) })
    }
}

impl FileSystemSetup for Ext2 {
    fn try_init(dev: Arc<BlockDev>) -> Result<Option<Arc<Ext2>>> {
        let mut buffer = [0 as u8; 1024];

        // Read the superblock
        // TODO I'm not clear on the buffer size still
        dev.read(1, &mut buffer)?;
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
            info: Info {
                lba_offset: { sb.superblock_index } as usize,
                block_size: 1024 << { sb.block_size_log2 },
            },
            sb: sb.clone(),
            block_dev: dev.clone(),
        };
        Ok(Some(Arc::new(fs)))
    }
}

impl Filesystem for Arc<Ext2> {
    // TODO handle other ext2 versions
    fn get_root_inode(&self) -> Result<Inonum> {
        // TODO this will be offsetted by -1, maybe not clear enough
        let rootindex: Inonum = 3;
        Ok(rootindex)
    }

    fn read_inode(&self, inode: Inonum) -> Result<Vnode> {
        // self.driver.read
        let raw_inode = self.get_inode(inode)?;
        let kind = {
            if raw_inode.mode & 0x4000 != 0 {
                VnodeType::Dir
            } else if raw_inode.mode & 0x8000 != 0 {
                VnodeType::File
            } else {
                panic!("Unimplemented inode mode {:x}", { raw_inode.mode })
            }
        };

        Ok(Vnode {
            inode,
            uid: raw_inode.uid,
            gid: raw_inode.guid,
            mode: raw_inode.mode,
            kind,
            ops: Arc::new(Ext2NodeOps { fs: self.clone() }),
        })
    }
}

pub struct Ext2NodeOps {
    fs: Arc<Ext2>,
}
pub struct Ext2File {}

impl NodeOps for Ext2NodeOps {
    fn open(&self, node: &Vnode, dent: &Arc<Dentry>) -> Result<File> {
        let inode = self.fs.get_inode(node.inode)?;
        let ops = match node.kind {
            VnodeType::FIFO => todo!(),
            VnodeType::Char => todo!(),
            VnodeType::Dir => Box::new(Ext2Dir {
                inum: node.inode,
                inode,
                block_index: 0,
                fs: self.fs.clone(),
                blocks: inode.blocks(self.fs.block_dev.clone()),
                buff: DirBuff { offset: 0, curr: 0, data: [0; 1024]}
            }),
            VnodeType::Block => todo!(),
            VnodeType::File => todo!(),
            VnodeType::Symlink => todo!(),
            VnodeType::Socket => todo!(),
        };


        Ok(File {
            dentry: dent.clone(),
            pos: 0,
            ops,
        })
    }
}


impl FileOps for Ext2File {}

pub struct DirBuff {
    offset: usize,
    curr: Lba,
    data: [u8; 1024],
}

/// Interface implmenting FileOps for the vfs
pub struct Ext2Dir {
    inum: Inonum,
    inode: Inode,
    block_index: usize,
    fs: Arc<Ext2>,
    blocks: InoBlocks,
    buff: DirBuff
}

impl FileOps for Ext2Dir {

    fn open(&mut self) -> Result<()> {
        Ok(())
    }

    fn readdir(&mut self) -> Result<Option<Dirent>> {
        // get lba from linear block index
        let r = self.blocks.get_lba(&self.inode, self.block_index)?;
        if r.is_none() {return Ok(None)}
        let lba = r.unwrap();
        // If the lba is not the one cached
        if lba != self.buff.curr {
            // fetch the data
            self.fs.block_dev.read(lba, &mut self.buff.data)?;
            self.buff.curr = lba;
            self.buff.offset = 0;
        }

        let dentry: &Ext2Dentry = unsafe {
            &*(&self.buff.data[self.buff.offset] as *const u8 as *const Ext2Dentry)
        };

        let mut dirent = Dirent {
            inode: dentry.inode as Inonum,
            name: [0 as u8; NAME_MAX],
            size: dentry.size as usize
        };
        dirent.name.clone_from_slice(dentry.name().unwrap().as_bytes());
        // offset by the entry size
        self.buff.offset += dirent.size;
        // If end of dirent for current block, go to next block
        if self.buff.offset == 1024 {
            self.block_index += 1;
        }
        return Ok(Some(dirent));
    }
}
