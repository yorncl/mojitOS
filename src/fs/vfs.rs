use alloc::boxed::Box;
use alloc::sync::Arc;
use core::cell::RefCell;

use super::block::BlockDriver;

// use to compose other filesystem
pub struct Info {
    // Offset to the starting LBA for the volume
    // The
    pub lba_offset: usize,
    pub block_size: usize,
}

pub enum Error {
    DONTWANNA,
}

#[repr(u8)]
#[allow(dead_code)]
enum InodeType {
    FIFO,
    Char,
    Dir,
    Block,
    File,
    Symlink,
    Socket,
}

pub struct Inode {
    kind: InodeType,
}

pub struct Dentry {
    inode: Arc<Inode>,
}

// TODO filesystem error management
pub trait FilesystemInit {
    fn match_superblock(buffer: &[u8]) -> bool;
    fn init(
        abs_lba_start: usize,
        abs_lba_super: usize,
        driver: Arc<dyn BlockDriver>,
    ) -> Result<Arc<Self>, ()>;
}

// Every filestystem will expose this API
// It is the interface between them and the VFS layer
pub trait Filesystem {
    fn get_root(&self) -> Result<Inode, ()>;
    // Reads the superblock and check magic number

    // fn open(&self, path: &str) -> Result<Inode, Error> {
    // }
    // fn open_dir(&self, path: &str) -> Result<Inode, Error> {
    // }
    // fn write_block() -> Result<(),()> {}
}


// TODO refactor to be more efficient
// optimize heap access, like a pointer ?
type FsVec = Vec<Arc<dyn Filesystem>>;
use alloc::vec::Vec;
static mut FILESYSTEMS: FsVec = vec![];
pub fn get_filesystems() -> &'static mut FsVec {
    unsafe {&mut FILESYSTEMS}
}

pub struct Filed {}
pub struct Dird {}

static mut KERN_ROOT_INODE: Inode = Inode {
    kind: InodeType::Dir,
};

pub fn path_walk(path: &str) -> Result<Inode, Error> {
    if path.len() == 0 {
        return Err(Error::DONTWANNA);
    }
    let bytes = path.as_bytes();
    if bytes[0] != b'/' {
        // TODO
        panic!("path_walk doesn't handle relative paths");
    }

    Err(Error::DONTWANNA)
    // Ok(Inode {
    //     kind:
    // })
}

// Takes the root filesystem
pub fn mount_kern_root(rootfs: Arc<dyn Filesystem>) -> Result<(), Error> {
    unsafe {
        KERN_ROOT_INODE = rootfs.get_root().unwrap();
    }
    Ok(())
}

pub fn open(path: &str) -> Result<Filed, Error> {
    Err(Error::DONTWANNA)
}

pub fn opendir() -> Result<Dird, Error> {
    Err(Error::DONTWANNA)
}

pub fn init() {
    //
}
