use core::arch::asm;


#[repr(C)]
pub struct GdtEntry {
    limit_low: u16,
    base_low: u16,
    base_mid: u8,
    access: u8,
    flags_limit_high: u8,
    base_high: u8,
}

const NENTRIES : usize = 5;

static GDT : [GdtEntry; NENTRIES] = [
    // null descriptor 
    GdtEntry {
        limit_low: 0,
        base_low: 0,
        base_mid: 0,
        access: 0,
        flags_limit_high: 0,
        base_high: 0,
    },
    // kernel code segment
    GdtEntry {
        limit_low: 0xffff,
        base_low: 0,
        base_mid: 0,
        access: 0x9a,
        flags_limit_high: 0xcf,
        base_high: 0,
    },
    // kernel data segment
    GdtEntry {
        limit_low: 0xffff,
        base_low: 0,
        base_mid: 0,
        access: 0x92,
        flags_limit_high: 0xcf,
        base_high: 0,
    },
    // user code segment
    GdtEntry {
        limit_low: 0xffff,
        base_low: 0,
        base_mid: 0,
        access: 0xfa,
        flags_limit_high: 0xcf,
        base_high: 0,
    },
    // user data segment
    GdtEntry {
        limit_low: 0xffff,
        base_low: 0,
        base_mid: 0,
        access: 0xf2,
        flags_limit_high: 0xcf,
        base_high: 0,
    },
    // task state segment // TODO add tss ?
    // GdtEntry {
    //     limit_low: 0xffff,
    //     base_low: 0,
    //     base_mid: 0,
    //     access: 0x89,
    //     flags_limit_high: 0x0,
    //     base_high: 0,
    // },
];

#[repr(C)]
struct GdtPtr {
    limit : u16,
    base : u32,
}

static mut GDT_PTR : GdtPtr = GdtPtr {limit : 0, base : 0};

// pub fn init() -> GDT
// {
//     GDT
//     {
//     }
// }

pub fn load()
{
    let b : u32 = &GDT as *const _ as u32;
    unsafe {
        GDT_PTR.limit = core::mem::size_of::<[GdtEntry; NENTRIES]>() as u16;
        GDT_PTR.base = b;
    };
    unsafe {
        asm!("cli","lgdt [{}]",
             in(reg) &GDT_PTR,
            options(nostack, preserves_flags)
        );
    }
}


