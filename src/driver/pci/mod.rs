use crate::klog;
use alloc::vec::Vec;
use config::PCIEndpointConfig;
use crate::klib::lock::RwLock;

pub mod config;
mod id;

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
    pub kind: PCIType,
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

pub static PCI_DEVICES: RwLock<Vec<PCIDevice>> = RwLock::new(vec![]);

pub fn init() {
    // TODO check ACPI tables for PCI support, it's assumed there
    id::enumerate();
    let devices = PCI_DEVICES.read().unwrap();
    klog!("{} PCI devices detected", devices.len());
    for dev in devices.iter() {
        klog!("{:?}", dev);
    }
}
