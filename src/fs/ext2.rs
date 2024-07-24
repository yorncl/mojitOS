use alloc::rc::Rc;

use super::{vfs, Filesystem};

struct Ext2 {
    // moutnpoint: Rc<vfs::Dentry>
}

impl Filesystem for Ext2 {
}
