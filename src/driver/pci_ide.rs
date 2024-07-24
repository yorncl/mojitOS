use crate::io;

use crate::arch::pci::PCIDevice;

use crate::fs::block;

use crate::klog;


pub struct IDEDev {
}


impl block::BlockDriver for IDEDev {

    fn read_block(&self, lba: usize) {
    }

}

impl IDEDev {
    pub fn write_sector() {
        // TODO should flush by ending 0xE7 command after write
    }
}

use alloc::boxed::Box;
pub(crate) fn probe_controller(pci_dev: &PCIDevice) -> Option<Box<IDEDev>> {
    klog!("Probing IDE controller");

    let caps = pci_dev.h.progif;

    // TODO hmmm I should probably return errors
    if caps & 1 != 0 || caps & 1 << 2 != 0 {
        panic!("IDE driver only supports compatibility mode");
    }
    if caps & 1 << 7 == 0 {
        panic!("IDE driver needs bus mastering to function properly");
    }

    let bar = pci_dev.get_bar(4);
    klog!("IDE bar 4 : 0x{:x}", bar);

    klog!("IO por status 0x{:x}", io::inb(bar as u16 + 0x2));

    loop{}
    None
}
