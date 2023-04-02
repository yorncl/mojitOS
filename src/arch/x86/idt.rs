use crate::VGA_INSTANCE;
use core::fmt::Write;
use core::arch::asm;

#[repr(C)]
#[derive(Debug, Clone, Copy)]
struct IdtEntry {
    offset_low: u16,
    selector: u16,
    zero: u8,
    flags: u8,
    offset_high: u16,
}

static mut IDT : [IdtEntry; 256] = [IdtEntry { offset_low: 0, selector: 0, zero: 0, flags: 0, offset_high: 0 }; 256];

struct IDTDESC {
    size: u16,
    offset: u32,
}

static mut IDTR : IDTDESC = IDTDESC { size: 0, offset: 0 };

pub fn test_handler()
{
    unsafe {
        write!(VGA_INSTANCE.as_mut().unwrap(), "Keyboard interrupt\n");
    }
}

pub fn setup()
{
    unsafe {
        IDT[0x21].offset_low = test_handler as u32 as u16;
        IDT[0x21].offset_high = (test_handler as u32 >> 16) as u16;
        IDT[0x21].selector = 0x08;
        IDT[0x21].flags = 0x8e;
        IDT[0x21] = IdtEntry { offset_low: 0, selector: 0, zero: 0, flags: 0, offset_high: 0 }; // TODO implement locks 
        IDTR.size = (IDT.len() * core::mem::size_of::<IdtEntry>() - 1) as u16;
        IDTR.offset = &IDT as *const _ as u32;
        // print IDTR


        asm!(
            "lidt [{0}]",
            "sti",
            in(reg) &IDTR,
            options(nostack, preserves_flags) // TODO is it necessary ?
        );
    }
}
