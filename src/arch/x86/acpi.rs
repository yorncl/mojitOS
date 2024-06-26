use crate::x86::iomem;
use crate::{klog, KERNEL_OFFSET};
use crate::memory::vmm;


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
    ptr_sdt: *const usize
}


#[repr(C, packed)]
struct ACPISDTHeader {
    signature: [u8; 4],
    length: u32,
    revision: u8,
    checksum: u8,
    oem_id: [u8; 6],
    oem_table_id: [u8; 8],
    oem_revision: u32,
    creator_id: u32,
    creator_revision: u32,
}


// TODO is this the way ?
#[inline(always)]
fn phys_to_virt(base: usize) -> usize {
    base + KERNEL_OFFSET
}

fn search_rsdp() -> Result<usize, ()> {
    // search for signature from 0x000E0000 to 0x000FFFFF
    // we're not in real mode
    unsafe {
        // TODO beurk l'offset encore une fois
        let mut ptr: usize = phys_to_virt(0xE0000);
        while ptr < phys_to_virt(0x000FFFFF) {
            if *(ptr as *const u64) == RSDP_SIGNATURE {
                return Ok(ptr as usize)
            }
            ptr += 0x10;
        }
    }
    Err(())
}

pub fn init() -> Result<(), &'static str> {

    let rsdp_t: &XSDPT;
    // let rsdp: &ACPISDTHeader;
    match search_rsdp() {
        Ok(address) => {unsafe {rsdp_t = &*(address as *const XSDPT);}}, 
        // TODO check the checksum
        Err(()) => panic!("Didn't find RSDP")
    }
    // // TODO handle version > 1.0
    if rsdp_t.revision == 0 {
        let rsdt_addr = {rsdp_t.rsdt_address} as usize;
        klog!("RSDT Address : {:x}", rsdt_addr);
        klog!("RSDT virtual Address : {:x}", phys_to_virt(rsdt_addr));
        klog!("RSDT length : {}", {rsdp_t.length});
        klog!("RSDT checksum : {}", rsdp_t.checksum);

        if vmm::mapper::virt_to_phys_kernel(phys_to_virt(rsdt_addr)) != None {
            return Err("ACPI tables are already mapped when they should not be");
        }
        unsafe {
            let rsdt: &RSDT;
            match iomem::remap_phys(rsdt_addr, core::mem::size_of::<RSDT>()) {
                Ok(addr) => rsdt = &*(addr as *const RSDT),
                Err(msg) => return Err(msg)
            }
            klog!("POINTER TO RSDT CHECK '{}'", core::str::from_utf8(&rsdt.h.signature).unwrap());
        }

    }
    else {
        panic!("ACPI version too high, not supported!");
    }
    Ok(())
}
