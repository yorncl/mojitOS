use crate::dbg;
use crate::memory::vmm::{self, mapper};
use crate::x86::iomem;

use super::apic;
use core::mem::size_of;
use core::ptr::addr_of;

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
                "Found RSDP, SIGNATURE {}",
                core::str::from_utf8(&rsdp.signature).unwrap()
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

    // Physical address
    let rsdt_addr = { rsdp.rsdt_address } as usize;
    dbg!("rsdt address {:x}", rsdt_addr);


    // // TODO used to make sense before the linear mapping
    // if vmm::mapper::virt_to_phys_kernel(vmm::mapper::phys_to_virt(rsdt_addr).unwrap()) == None {
    //     return Err("ACPI tables aren't mapped");
    // }
    unsafe {
        // MMIO remap the zone where the RSDT is
        let rsdt: &RSDT = &*(iomem::remap_phys(rsdt_addr, size_of::<RSDT>())? as *const RSDT);
        // let rsdt: &RSDT = &*(rsdt_addr as *const RSDT);

        dbg!("Rsdt {:p}", rsdt);

        // MMIO remap the zone where the entries pointed by the RSDT are
        dbg!(
            "Rsdt len {} vs {}",
            { rsdt.h.length },
            size_of::<ACPISDTHeader>()
        );

        dbg!("SIGNATURE OF HEADER {}", core::str::from_utf8(&rsdt.h.signature).unwrap());
        // subtracting size of header to get numbers of entries
        let nentries =
            (rsdt.h.length - size_of::<ACPISDTHeader>() as u32) / size_of::<usize>() as u32;

        let tables_ptr = iomem::remap_phys(
            rsdt.ptr_sdt as usize,
            nentries as usize * size_of::<ACPISDTHeader>(),
        )? as *const usize;

        dbg!("Rsdt {:p}", rsdt);

        // recompute the offsets in the table
        let base_phys: usize = rsdt.ptr_sdt as usize;
        // tables_ptr is the virtual pointer to where the tables are actually stored
        let base_virt = tables_ptr as usize;
        // ptr is the pointer to the entries of the array
        // each entry will then point to the physical location of the corresponding table
        let mut ptr = addr_of!(rsdt.ptr_sdt);
        for _i in 0..nentries {
            let entry = base_virt + (*ptr as usize - base_phys);
            let header = &*(entry as *const ACPISDTHeader);
            match core::str::from_utf8(&header.signature).unwrap() {
                //MADT table
                "APIC" => {
                    apic::parse_madt(header);
                }
                _ => {
                    // TODO Unsupported tables
                }
            }
            ptr = ptr.offset(1);
        }
    }
    Ok(())
}
