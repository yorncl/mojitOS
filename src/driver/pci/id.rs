// Enumerate and identify the devices
use crate::arch::io;

use super::{PCIDevice, PCIType, config::PCIEndpointConfig};
use crate::arch::io::port::{PCICONFIG_ADDRESS, PCICONFIG_DATA};

pub fn build_address(bus_num: u8, dev_num: u8, fn_num: u8, reg_off: u8) -> u32 {
    assert!(reg_off & 0x3 == 0); // only address 32 byte chunks
    let mut address: u32 = 1 << 31; // type 1 configuration
    address |= (bus_num as u32) << 16;
    address |= (dev_num as u32) << 11;
    address |= (fn_num as u32) << 8;
    address |= reg_off as u32;
    address
}

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

fn identify_class(conf: &PCIEndpointConfig) -> PCIType {
    return match conf.class {
        0x101 => {
            PCIType::IDE
        }
        0x105 => {
            PCIType::ATA
        }
        0x106 => {
            PCIType::SATA
        }
        _ => {
            PCIType::Unsupported
        }
    }
}

fn enumerate_functions(bus_num: u8, dev_num: u8) {
    for fn_num in 0..=8 {
        if device_exist(bus_num, dev_num, fn_num) {

            let addr = build_address(bus_num, dev_num, fn_num, 0);
            let conf = PCIEndpointConfig::from_io_space(addr);

            if conf.header_type & 0x3 != 0 {
                panic!("Unimplemented PCI header type: {}", conf.header_type);
            }

            // New device
            let new_dev = PCIDevice {
                config: conf,
                bus_num,
                dev_num,
                fn_num,
                kind: identify_class(&conf),
            };

            let mut pci_devs = super::PCI_DEVICES.write().unwrap();
            pci_devs.push(new_dev);
            drop(pci_devs);
            // Multi-function device
            // bit 7 checks if this is a multi-funciton device
            // if not break the loop
            if new_dev.config.header_type & 0x80 == 0 {
                break;
            }
        }
    }
}

pub fn enumerate() {
    for bus_num in 0..=255 {
        for dev_num in 0..32 {
            if device_exist(bus_num, dev_num, 0) {
                enumerate_functions(bus_num, dev_num);
            }
        }
    }
}


