use crate::dbg;
use crate::memory::vmm::{self, mapper};

use super::apic;
use core::mem::size_of;
use core::ptr::addr_of;
use core::ptr;

static RSDP_SIGNATURE: u64 = u64::from_le_bytes(*b"RSD PTR ");

#[repr(C, packed)]
struct XSDPT {
    pub signature: [u8; 8],
    pub checksum: u8,
    pub oem_id: [u8; 6],
    pub revision: u8,
    pub rsdt_address: u32, // deprecated since version 2.0

    pub length: u32,
    pub xsdt_address: u64,
    pub extended_checksum: u8,
    pub reserved: [u8; 3],
}

#[repr(C, packed)]
struct RSDT {
    h: ACPISDTHeader,
    ptr_sdt: *const usize,
}

#[repr(C, packed)]
pub struct ACPISDTHeader {
    pub signature: [u8; 4],
    pub length: u32,
    pub revision: u8,
    pub checksum: u8,
    pub oem_id: [u8; 6],
    pub oem_table_id: [u8; 8],
    pub oem_revision: u32,
    pub creator_id: u32,
    pub creator_revision: u32,
}

fn search_rsdp() -> Result<usize, ()> {
    // search for signature from 0x000E0000 to 0x000FFFFF (mapped in kernel space)
    unsafe {
        let mut ptr: usize = vmm::mapper::phys_to_virt(0xE0000).ok_or(())?;
        while ptr < vmm::mapper::phys_to_virt(0x000FFFFF).ok_or(())? {
            if *(ptr as *const u64) == RSDP_SIGNATURE {
                return Ok(ptr as usize);
            }
            ptr += 0x10;
        }
    }
    Err(())
}

// TODO str signature feels ugly
pub fn init() -> Result<(), &'static str> {
    let rsdp: &XSDPT;
    match search_rsdp() {
        Ok(address) => {
            unsafe {
                rsdp = &*(address as *const XSDPT);
            }
            dbg!(
                "Found RSDP, SIGNATURE: \"{}\", RSDT ADDRESS: {:x}",
                core::str::from_utf8(&rsdp.signature).unwrap(),
                {rsdp.rsdt_address}
            );
        }
        // TODO check the checksum
        Err(()) => panic!("Didn't find RSDP"),
    }


    // TODO handle version > 1.0
    if rsdp.revision != 0 {
        panic!(
            "ACPI version not supported, rsdt revision = {}",
            rsdp.revision
        );
    }


    unsafe {
        // Physical address
        let rsdt_addr = mapper::phys_to_virt(rsdp.rsdt_address as usize).unwrap() as usize;
        dbg!("---- >rsdt address {:x}", rsdt_addr);
        let rsdt: &RSDT = &*(rsdt_addr as *const RSDT);

        dbg!("SIGNATURE OF RSDT HEADER: \"{}\"", core::str::from_utf8(&rsdt.h.signature).unwrap());
        // calculating number of entries
        let nentries =
            (rsdt.h.length - size_of::<ACPISDTHeader>() as u32) / size_of::<usize>() as u32;

        // get pointer to array of addresses that is at the end of the rsdt structure
        let mut table_ptr:usize = addr_of!(rsdt.ptr_sdt) as usize;
        dbg!("table_ptr = {:x}", table_ptr);

        for _i in 0..nentries {
            dbg!("=========== ");
            let entry_addr = mapper::phys_to_virt(ptr::read_unaligned(table_ptr as *const u32) as usize).unwrap() as *const ACPISDTHeader;
            let entry = &*(entry_addr);
            match core::str::from_utf8(&entry.signature).unwrap() { 
                //MADT table
                "APIC" => {
                    apic::parse_madt(entry);
                }
                _ => {
                    // TODO Unsupported tables
                }
            }
            table_ptr += 4;
        }
    }
    Ok(())
}
