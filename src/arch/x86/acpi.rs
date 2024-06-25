use crate::{klog, KERNEL_OFFSET, kprint};


static RSDP_SIGNATURE: u64 = u64::from_le_bytes(*b"RSD PTR ");


fn search_rsdp() -> Result<usize, ()> {
    // search for signature from 0x000E0000 to 0x000FFFFF
    // we're not in real mode
    unsafe {
        // TODO beurk l'offset encore une fois
        let mut ptr: usize = 0xE0000 + KERNEL_OFFSET;
        while ptr < 0x000FFFFF + KERNEL_OFFSET {
            if *(ptr as *const u64) == RSDP_SIGNATURE {
                return Ok(ptr as usize)
            }
            // kprint!("{}", *(ptr as *const char));
            ptr += 0x10;
        }
    }
    Err(())
}


pub fn init() {
    match search_rsdp() {
        Ok(value) => klog!("Found RSDP at address: {:x}", value), 
        Err(()) => panic!("Didn't find RSDP")
    }
}
