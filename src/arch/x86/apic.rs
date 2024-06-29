use crate::{klog, PAGE_SIZE};
use crate::x86::iomem;

use core::mem::size_of;
use super::util;

use super::acpi::ACPISDTHeader;


#[repr(usize)]
#[allow(dead_code)]
enum RegId {
    Spurious = 0xf0, // interrupts which have no source
    Eoi = 0xb0,
    TimerLvt = 0x320, // timer and local interrupts
    ApicId = 0x20
}

// We need to store the address of APIC IO kj
struct LAPIC {
    address: u32,
    flags: u32
}

#[repr(C, packed)]
struct EntryHeader {
    etype: u8,
    // includes the length of the entry and the entry header
    length: u8
}

#[repr(C, packed)]
struct EntryIOAPIC {
    id: u8,
    _reserved: u8,
    address: u32,
    // Global interrupt base
    gib: u32
}

#[repr(C, packed)]
struct EntrySourceOverride {
    bus: u8,
    irq: u8,
    // Global System Interrupt
    gsi: u32,
    flags: u16
}

static mut IOAPIC_REMAP: usize = 0x0;
#[inline(always)]
fn ioapic_read_reg(reg: u32) -> u32{
    unsafe {
        // select register in IOREGSEL
        let mut ptr = IOAPIC_REMAP as *mut u32;
        core::intrinsics::volatile_store::<u32>(ptr, reg as u32);
        //  Offset to IOWIN
        ptr = (IOAPIC_REMAP + 0x10) as *mut u32;
        core::intrinsics::volatile_load::<u32>(ptr)
    }
}
#[inline(always)]
fn ioapic_write_reg(reg: u32, value: u32) {
    unsafe {
        // select register in IOREGSEL
        let mut ptr = IOAPIC_REMAP as *mut u32;
        core::intrinsics::volatile_store::<u32>(ptr, reg as u32);
        //  Offset to IOWIN
        ptr = (IOAPIC_REMAP + 0x10) as *mut u32;
        core::intrinsics::volatile_store::<u32>(ptr, value);
    }
}

static mut LAPIC_REMAP: usize = 0x0;
#[inline(always)]
fn lapic_read_reg(offset: RegId) -> u32{
    unsafe {*((LAPIC_REMAP + offset as usize) as *const u32)}
}
#[inline(always)]
fn lapic_write_reg(offset: RegId, value: u32) {
    unsafe {*((LAPIC_REMAP + offset as usize) as *mut u32) = value}
}


pub fn end_of_interrupt() {
    lapic_write_reg(RegId::Eoi, 0);
}

// TODO only for test purposes
pub fn enable_ioapic_interrupts() {

    let low = ioapic_read_reg(0x12);
    let high = ioapic_read_reg(0x13);
    klog!("IOAPIC INT 1 Low {:x}", low);
    klog!("IOAPIC INT 1 High {:x}", high);
    // read apic id
    let id = lapic_read_reg(RegId::ApicId);
    // TODO handle the destination correctly
    klog!("LAPIC id {:x}", id);


    ioapic_write_reg(0x12, 42);

    // setting 11th bit of apic base to enable apic TODO sould be done
    // let (low, high) = util::readmsr(0x1b);
    // klog!("Low and high {} {}", low, high);

    // ioapic_read_reg(0x13,);
    // klog!("IOAPIC INT 1 Low {:x}", low);
    // klog!("IOAPIC INT 1 High {:x}", high);
}

pub fn enable_lapic() {
        let (low, _high) = util::readmsr(util::msrid::LOCAL_APIC_BASE);
        let base = low as u32 & !0xfff;
        klog!("MSR READING FOR LAPIC {:x}", base);
        match iomem::remap_phys(base as usize, PAGE_SIZE) {
            Ok(ptr) => {
                unsafe {
                    LAPIC_REMAP = ptr;
                }
            },
            Err(msg) => panic!("Could not remap IOAPIC base: {}", msg)
        }
        // Setting the last entry in IDT for the spurious interrupts with 0xff
        // setting the 8th bit to enable the local APIC
        let r = lapic_read_reg(RegId::Spurious) | 0xff | (1 << 8);
        lapic_write_reg(RegId::Spurious, r); 

}

// impl Apic {
// }

// Parse the MADT table
// Find the IO APIC address
pub fn parse_madt(address: *const ACPISDTHeader) {
    klog!("APIC init start with addres {:p}", address);

    unsafe {
        let lapic : *const LAPIC = (address as usize + size_of::<ACPISDTHeader>()) as *const LAPIC;
        klog!("LAPIC adress {:x} flags :{}", (*lapic).address, (*lapic).flags);

        let mut eptr = (lapic as usize + size_of::<LAPIC>()) as *const EntryHeader;
        let end = (*address).length as usize + address as usize;
        while (eptr as usize) < end {
            let entry_header = &*eptr;
            let entry_addr = eptr as usize + size_of::<EntryHeader>();
            match entry_header.etype {
                // 0x00 => {
                //     // Local APIC

                // }
                0x01 => {
                    // TODO manage this case
                    if IOAPIC_REMAP != 0 {
                        panic!("More than 1 IOAPIC detected!")
                    }
                    let ioptr: &EntryIOAPIC = &*(entry_addr as *const EntryIOAPIC);
                    klog!("IOAPIC id {:x}", {ioptr.id});
                    klog!("IOAPIC base {:x}", {ioptr.address});
                    klog!("IOAPIC gib {:x}", {ioptr.gib});
                    // IO APIC
                    match iomem::remap_phys({ioptr.address} as usize, PAGE_SIZE) {
                        Ok(ptr) => {
                            IOAPIC_REMAP = ptr;
                        },
                        Err(msg) => panic!("Could not remap IOAPIC base: {}", msg)
                    }
                },
                0x02 => {
                    let source: &EntrySourceOverride = &*(entry_addr as *const EntrySourceOverride);
                    klog!("Source override irq {} gsi {}", {source.irq}, {source.gsi});
                },
                _value => {
                    // TODO implement missing entries
                }
            }
            eptr = (eptr as usize + entry_header.length as usize) as *const EntryHeader;
        }
        enable_lapic();
        enable_ioapic_interrupts();
    }
    klog!("APIC init end");
}




