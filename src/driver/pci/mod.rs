use crate::io::PortIO;
use crate::klog;

mod id;

pub mod config;

use config::PCIEndpointConfig;


#[derive(Debug, Copy, Clone)]
pub enum PCIType {
    Unsupported,
    IDE,
    ATA,
    SATA,
}


#[allow(dead_code)]
#[derive(Copy, Clone)]
pub struct PCIDevice {
    // TODO we should probably manage PCI-to-PCI and PCI-to-CardBus bridges too
    pub config: PCIEndpointConfig,
    pub bus_num: u8,
    pub dev_num: u8,
    pub fn_num: u8,
    pub kind: PCIType
}

impl PCIDevice {
}

impl core::fmt::Debug for PCIDevice {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        let mut r = f.debug_struct("PCIDevice");
        r.field("io", &self.bus_num)
        .field("dev", &self.dev_num)
        .field("fn", &self.fn_num)
        .field("kind", &self.kind);
        if let PCIType::Unsupported = self.kind {
            r.field("class", &format_args!("0x{:x}", self.config.class));
        }
            r.finish()
    }
}

// TODO put behind lock probably
use alloc::vec::Vec;

static mut PCI_DEVICES: Vec<PCIDevice> = vec![];

// TODO PUT IN A CELL OR LOCK DO SOMETHING
#[inline]
pub fn get_devices() -> &'static mut Vec<PCIDevice> {
    unsafe {return &mut PCI_DEVICES}
}

pub fn init() {
    // TODO check ACPI tables for PCI support, it's assumed there
    id::enumerate();
    klog!("{} PCI devices detected", get_devices().len());
    for dev in get_devices().iter() {
        klog!("{:?}", dev);
    }
}

