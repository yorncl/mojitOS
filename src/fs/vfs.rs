use super::block::Partition;
use crate::dbg;
use crate::error::{Result, codes::*};
use alloc::sync::Arc;

// use to compose other filesystem
pub struct Info {
    // Offset to the starting LBA for the volume
    // The
    pub lba_offset: usize,
    pub block_size: usize,
}

#[repr(u8)]
#[allow(dead_code)]
enum VnodeType {
    FIFO,
    Char,
    Dir,
    Block,
    File,
    Symlink,
    Socket,
}

pub struct Mountpoint {
    path: Path,
    fs: Arc<dyn Filesystem>,
}

pub struct Path {
    buff: Vec<u8>,
}
impl Path {
    /// Creates a new Path object from a string slice
    pub fn new(path: &str) -> Path {
        Path {
            buff: Vec::from_iter(path.as_bytes().iter().copied()),
        }
    }

    #[inline]
    pub fn len(&self) -> usize {
        self.buff.len()
    }
    /// Iterate through paths components
    pub fn component_iter(&self) -> core::iter::Skip<core::str::Split<'_, impl Fn(char) -> bool>> {
        let split = core::str::from_utf8(&self.buff)
            .unwrap()
            .split(|c: char| c == '/');
        // TODO ugh doesn't feel good
        if self.buff[0] == b'/' {
            return split.skip(1);
        }
        split.skip(0)
    }

    // TODO will not work on apths containing dots
    pub fn absolute(&self) -> bool {
        self.buff[0] == b'/'
    }
}
// TODO are those types sensible ?
// This is the virtual inode type
#[allow(dead_code)]
pub struct Vnode {
    // Inode number, unique identifier on the target filesystem
    inode: u32,

    uid: u32,
    gid: u32,
    mode: u32,

    kind: VnodeType,

    // TODO add timestamps
    fops: Arc<dyn FileIO>
    
}

// Directory entry, points to a single inode
#[allow(dead_code)]
pub struct Dentry {
    vnode: Arc<Vnode>,
    path: Path,
}

// File descriptor structure
#[allow(dead_code)]
pub struct File {
    dentry: Arc<Dentry>,
    f_pos: u64,
}

impl File {
    // pub fn read(&self) {
    //     // call driver
    //     self.dentry.vnode.driver.read();
    // }
}

pub type Inonum = u64;

// Every filestystem will expose this API
// It is the interface between them and the VFS layer
#[allow(dead_code)]
pub trait Filesystem {
    fn get_root_inode(&self) -> Result<Inonum>;
    fn read_inode(&self, inode: Inonum) -> Result<Vnode>;
    fn read(&self) -> Result<usize>;
}

pub trait FileSystemSetup {
    fn try_init(driver: Arc<Partition>) -> Result<Option<Arc<Self>>>;
}

// TODO refactor to be more efficient
// optimize heap access, like a pointer ?
type FsVec = Vec<Arc<dyn Filesystem>>;
use alloc::vec::Vec;
static mut FILESYSTEMS: FsVec = vec![];
pub fn get_filesystems() -> &'static mut FsVec {
    unsafe { &mut *core::ptr::addr_of_mut!(FILESYSTEMS) }
}

// TODO lock?
static mut MOUNTPOINTS: Vec<Arc<Mountpoint>> = vec![];

// Takes the root filesystem
pub fn register_mount(path: &str, rootfs: Arc<dyn Filesystem>) -> Result<()> {
    unsafe {
        MOUNTPOINTS.push(Arc::new(Mountpoint {
            path: Path::new(path),
            fs: rootfs,
        }))
    }
    Ok(())
}

/// Will go through the list of mountpoints and find the longest match
/// If it doesn't find any match, the root mountpoint is returned
pub fn match_mountpoint(path: &Path) -> Arc<Mountpoint> {
    unsafe {
        let mut longest_match: usize = 0;
        let mut longest_index: usize = 0;
        for (i, mount) in MOUNTPOINTS.iter().enumerate() {
            if mount.path.len() > path.len() {
                continue;
            }
            let count = mount
                .path
                .component_iter()
                .zip(path.component_iter())
                .take_while(|(a, b)| a == b)
                .count();
            if count > longest_match {
                longest_match = count;
                longest_index = i;
            }
        }
        // TODO if the mountpoint at index 0 is not root, this will do something funky
        MOUNTPOINTS[usize::try_from(longest_index).unwrap()].clone()
    }
}

pub trait FileIO {
}

/// Takes a path and returns the corresponding node if any
pub fn walk_path_node(path: &Path) -> Result<Vnode> {
    dbg!("Here we are in this funciton");
    // TODO handle properly
    assert!(
        path.absolute() && !path.buff.contains(&b'.'),
        "Path is not absolute"
    );

    let mount = match_mountpoint(path);

    // TODO refactor
    // iterator starting at end of mountpoints path
    // TODO the -1 here is not clear enough, it's supposed to be for the root component that's
    // empty
    let mut components = path
        .component_iter()
        .skip(mount.path.component_iter().count() - 1).peekable().into_iter();

    let inode = mount.fs.get_root_inode()?;
    while let Some(c) = components.next() {
        let node: Vnode = mount.fs.read_inode(inode)?;

        // TODO check access here

        match node.kind {
            VnodeType::File => {
                // components left yet we're on a file, error
                if components.peek().is_none() {
                    return Err(ENOENT)
                }
            },
            VnodeType::Dir => {

            },
            VnodeType::FIFO => todo!(),
            VnodeType::Char => todo!(),
            VnodeType::Block => todo!(),
            VnodeType::Symlink => todo!(),
            VnodeType::Socket => todo!(),
        }
        // TODO manage access rights

        // empty component means no component
        if c.len() == 0 {
            continue;
        }
        dbg!("Component {}", c);
    }
    Err(EINVAL)
}

pub fn vfs_open(path: &str) -> Result<File> {
    let p = Path::new(path);
    let _node = walk_path_node(&p)?;

    Err(EINVAL)
}
