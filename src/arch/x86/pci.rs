use super::io;
use super::io::port::{PCICONFIG_ADDRESS, PCICONFIG_DATA};
use crate::klog;

use core::ptr::addr_of_mut;

#[allow(dead_code)]
#[repr(C, packed)]
#[derive(Default, Clone, Copy)]
pub struct PCIHeader {
    pub vendor: u16,
    pub id: u16,
    pub command: u16,
    pub status: u16,
    pub revid: u8,
    pub progif: u8,
    // subclass: u8,
    pub class: u16,
    pub cache_line_size: u8,
    pub latency_timer: u8,
    pub header_type: u8,
    pub bist: u8,
}

#[derive(Debug, Copy, Clone)]
pub enum PCIType {
    Unclassified,
    IDE,
    ATA,
    SATA,
}

#[allow(dead_code)]
#[derive(Copy, Clone)]
pub struct PCIDevice {
    pub header: PCIHeader,
    pub bus_num: u8,
    pub dev_num: u8,
    pub fn_num: u8,
    address: u32,
    pub kind: PCIType
}

impl PCIDevice {
    fn new(bus_num: u8, dev_num: u8, fn_num: u8) -> Self {
        PCIDevice {
            header: PCIHeader::default(),
            bus_num,
            dev_num,
            fn_num,
            address: build_address(bus_num, dev_num, fn_num, 0),
            kind: PCIType::Unclassified
        }
    }

    // TODO this API makes me sad
    pub fn get_bar(&self, index: u8) -> u32 {
        bus::read_reg(self.address, 0x10 + index * 4)
    }

    pub fn set_bar(&self, index: u8, value: u32) {
        bus::write_reg(self.address, 0x10 + index * 4, value);
    }

    pub fn enable_busmaster(&self) {

        klog!("COmmand {:b}", {self.header.command});
        let mut val = bus::read_reg(self.address, 0x4);
        klog!("COmmand           {:b}", val);
        val |= 1 << 2;
        klog!("COmmand  new      {:b}", val);
        bus::write_reg(self.address, 0x4, val);
        val = bus::read_reg(self.address, 0x4);
        klog!("COmmand after new {:b}", val);
    }
}

impl core::fmt::Debug for PCIDevice {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        let mut r = f.debug_struct("PCIDevice");
        r.field("bus", &self.bus_num)
        .field("dev", &self.dev_num)
        .field("fn", &self.fn_num)
        .field("kind", &self.kind);
        if let PCIType::Unclassified = self.kind {
            r.field("class", &format_args!("0x{:x}", {self.header.class}));
        }
            r.finish()
    }
}

mod bus {
    use super::*;

    pub fn device_exist(bus_num: u8, dev_num: u8, fn_num: u8) -> bool {
        io::outl(
            PCICONFIG_ADDRESS,
            build_address(bus_num, dev_num, fn_num, 0),
        );
        let idvendor = io::inl(PCICONFIG_DATA);
        // Check if id/vendor is all f, in which case the deviec does not exist (cf osdev wiki)
        if idvendor == u32::MAX {
            return false;
        }
        true
    }

    pub fn read_reg(device_base: u32, reg_off: u8) -> u32 {
        io::outl(PCICONFIG_ADDRESS, device_base | reg_off as u32);
        io::inl(PCICONFIG_DATA)
    }

    pub fn write_reg(device_base: u32, reg_off: u8, data: u32) {
        io::outl(PCICONFIG_ADDRESS, device_base | reg_off as u32);
        io::outl(PCICONFIG_DATA, data);
    }

    pub fn enumerate_bus(bus_num: u8) {
        for dev_num in 0..32 {
            if device_exist(bus_num, dev_num, 0) {
                enumerate_functions(bus_num, dev_num);
            }
        }
    }

    fn enumerate_functions(bus_num: u8, dev_num: u8) {
        for fn_num in 0..=8 {
            if device_exist(bus_num, dev_num, fn_num) {
                // New device
                let mut new_dev = PCIDevice::new(bus_num, dev_num, fn_num);
                new_dev.header = read_header(new_dev.address);

                investigate_device(&mut new_dev);

                unsafe {
                    PCI_DEVICES.push(Rc::new(new_dev.clone()));
                }
                // Multi-function device
                // bit 7 checks if this is a multi-funciton device
                // if not break the loop
                if new_dev.header.header_type & 0x80 == 0 {
                    break;
                }
            }
        }
    }

    fn read_header(device_base: u32) -> PCIHeader {
        let mut header: PCIHeader = PCIHeader::default();
        unsafe {
            let ptr = addr_of_mut!(header) as *mut u32;
            *ptr = read_reg(device_base, 0x0);
            *ptr.offset(1) = read_reg(device_base, 0x4);
            *ptr.offset(2) = read_reg(device_base, 0x8);
            *ptr.offset(3) = read_reg(device_base, 0xC);
        }
        header
    }
}

fn build_address(bus_num: u8, dev_num: u8, fn_num: u8, reg_off: u8) -> u32 {
    assert!(reg_off & 0x3 == 0); // only address 32 byte chunks
    let mut address: u32 = 1 << 31; // type 1 configuration
    address |= (bus_num as u32) << 16;
    address |= (dev_num as u32) << 11;
    address |= (fn_num as u32) << 8;
    address |= reg_off as u32;
    address
}

fn investigate_device(dev: &mut PCIDevice) {
    if {dev.header.header_type} & 0x3 != 0 {
        panic!("Unimplemented PCI header type: {}", {dev.header.header_type});
    }
    match { dev.header.class } {
        0x101 => {
            dev.kind = PCIType::IDE;
        }
        0x105 => {
            dev.kind = PCIType::ATA;
        }
        0x106 => {
            dev.kind = PCIType::SATA;
        }
        _ => {
            // klog!("PCI device controller not implemented : 0x{:x}{:x}", class, {h.subclass});
        }
    }
}

// TODO put behind lock probably
use alloc::rc::Rc;
use alloc::vec::Vec;
static mut PCI_DEVICES: Vec<Rc<PCIDevice>> = vec![];

pub fn collect_devices() {
    // TODO check ACPI tables for PCI support, it's assumed there
    for bus_num in 0..=255 {
        bus::enumerate_bus(bus_num);
    }
    unsafe {
        klog!("{} PCI devices detected", PCI_DEVICES.len());
        for dev in PCI_DEVICES.iter() {
            klog!("{:?}", dev);
        }
    }
}

// TODO PUT IN A CELL OR LOCK DO SOMETHING
pub fn get_devices() -> &'static Vec<Rc<PCIDevice>> {
    unsafe {return &PCI_DEVICES}

}
