use super::io;
use super::io::Port::{PCICONFIG_DATA, PCICONFIG_ADDRESS};
use crate::klog;

use core::ptr::addr_of_mut;
#[allow(dead_code)]
#[repr(u16)]
enum Class {
    // None
    NoVga = 0x0,
    // YesVga = 0x1,
    // Mass storage
    // SCSIBus = 0x10,
    IDE = 0x11,
    ATA = 0x15,
    SATA = 0x16,
}

impl Default for Class {
    fn default() -> Self {
       Class::NoVga
    }
}

#[allow(dead_code)]
#[repr(C, packed)]
#[derive(Default, Clone, Copy)]
struct PCIHeader {
    vendor: u16,
    id: u16,
    command: u16,
    status: u16,
    revid: u8,
    progif: u8,
    // subclass: u8,
    class: u16,
    cache_line_size: u8,
    latency_timer: u8,
    header_type: u8,
    bist: u8,
}

fn device_exist(bus_num: u8, dev_num: u8, fn_num: u8) -> bool {
    io::outl(PCICONFIG_ADDRESS, build_address(bus_num, dev_num, fn_num, 0));
    let idvendor = io::inl(PCICONFIG_DATA);
    // Check if id/vendor is all f, in which case the deviec does not exist (cf osdev wiki)
    if idvendor == u32::MAX {
        return false
    }
    true
}

fn read_reg(device_base: u32, reg_off: u8) -> u32 {
    io::outl(PCICONFIG_ADDRESS, device_base | reg_off as u32);
    io::inl(PCICONFIG_DATA)
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

fn build_address(bus_num: u8, dev_num: u8, fn_num: u8, reg_off: u8) -> u32 {
    assert!(reg_off & 0x3 == 0); // only address 32 byte chunks
    let mut address: u32 = 1 << 31; // type 1 configuration
    address |= (bus_num as u32) << 16; 
    address |= (dev_num as u32) << 11; 
    address |= (fn_num as u32) << 8; 
    address |= reg_off as u32;
    address
}


fn investigate_device(h: &PCIHeader) {
    // match {h.header_type} & 0x3 {
    //     0x0 => {klog!("Header type 0x0")},
    //     0x1 => {klog!("Header type 0x1")},
    //     0x2 => {klog!("Header type 0x2")},
    //     _ => {klog!("-------------------- Unkown header type : {:x}", {h.header_type})},
    // }
    match {h.class} {
        0x101 => {
            klog!("ID:0x{:x} Vendor:0x{:x} class 0x{:x}", {h.id}, {h.vendor}, {h.class});
            klog!("+++++IDE controller detected, progif:0x{:b}", {h.progif});
        },
        0x105 => {
            klog!("ATA Controller detected");
        },
        0x106 => {
            klog!("SATA Controller detected");
        },
        _ => {
            // klog!("PCI device controller not implemented : 0x{:x}{:x}", class, {h.subclass});
        }
    }
}

fn enumerate_functions(bus_num: u8, dev_num: u8) {
    let mut h: PCIHeader;
    // Multi-function device
    for fn_num in 0..=8 {
        if device_exist(bus_num, dev_num, fn_num) {
            // klog!("==== PCI function device exist, bus:{} dev_num:{} fn_num:{}", bus_num, dev_num, fn_num);
            let device_base = build_address(bus_num, dev_num, fn_num, 0);
            h = read_header(device_base);
            investigate_device(&h);
        }
    }
}

fn enumerate () {
    let mut h: PCIHeader;
    for bus_num in 0..=255 {
        for dev_num in 0..32 {
            if device_exist(bus_num, dev_num, 0) {
                // klog!("==== PCI device exist, bus:{} dev_num:{} fn_num:{}", bus_num, dev_num, 0);
                let device_base = build_address(bus_num, dev_num, 0, 0);
                h = read_header(device_base);
                // bit 7 checks if this is a multi-funciton device
                if h.header_type & 0x80 != 0 {
                    enumerate_functions(bus_num, dev_num);
                }
                else {
                    investigate_device(&h);
                }
            }
        }
    }
}

pub fn init() {
    // TODO check ACPI tables for PCI support, it's assumed there
    enumerate();
}
