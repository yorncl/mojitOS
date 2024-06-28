use crate::KERNEL_OFFSET;
use crate::klog;
use super::util as util;

use super::acpi::ACPISDTHeader;


static mut APIC_BASE: usize = 0;
#[repr(usize)]
#[allow(dead_code)]
enum RegId {
    Spurious = 0xf0, // interrupts which have no source
    Eoi = 0xb0,
    TimerLvt = 0x320, // timer and local interrupts
    ApicId = 0x20
}

fn read_register(offset: RegId) -> u32{
    unsafe {*((APIC_BASE + offset as usize) as *const u32)}
}
fn write_register(offset: RegId, value: u32) {
    unsafe {*((APIC_BASE + offset as usize) as *mut u32) = value}
}


struct Apic {
}


impl Apic {
}

// Parse the MADT table
// Find the IO APIC address
pub fn parse_madt(address: *const ACPISDTHeader) {
    klog!("APIC init start with addres {:p}", address);
    // let msr = util::readmsr(util::msrid::LOCAL_APIC_BASE);

    // // TODO beurk ajouter le KERNEL OFFSET me rend malade
    // unsafe {
    //     APIC_BASE = (msr as usize >> 12) + KERNEL_OFFSET;

    //     // Setting the last entry in IDT for the spurious interrupts
    //     let mut r = read_register(RegId::Spurious) | 0xff;
    //     write_register(RegId::Spurious, r);


    //     // Setting the last entry in IDT for the spurious interrupt
    //     r = read_register(RegId::TimerLvt) | 0xff;
    //     write_register(RegId::Spurious, r);

    // }
    
    // klog!("APIC init end");
}




