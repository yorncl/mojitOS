use core::arch::asm;


#[allow(dead_code)]
pub mod msrid {
    pub const LOCAL_APIC_BASE: u32 = 0x1b;
}

pub fn readmsr(id: u32) -> (u32, u32) {

    let low: u32;
    let high: u32;
    unsafe {
        asm!(
            "rdmsr",
            "mov {high}, edx",
            "mov {low}, eax",
            in("ecx") id,
            low = out(reg) low,
            high = out(reg) high,
            out("eax") _,
            out("edx") _,
        )
    }
    (low, high)
}




