use crate::VGA_INSTANCE;
use core::fmt::Write;
use core::arch::asm;

#[repr(C)]
#[repr(packed)]
#[derive(Debug, Clone, Copy)]
struct IdtEntry {
    offset_low: u16,
    selector: u16,
    zero: u8,
    flags: u8,
    offset_high: u16,
}

static mut IDT : [IdtEntry; 256] = [IdtEntry { offset_low: 0, selector: 0, zero: 0, flags: 0, offset_high: 0 }; 256];

#[repr(C)]
#[repr(packed)]
struct IDTDESC {
    size: u16,
    offset: u32,
}

static mut IDTR : IDTDESC = IDTDESC { size: 0, offset: 0 };

#[no_mangle]
pub fn test_handler()
{
    unsafe {
        write!(VGA_INSTANCE.as_mut().unwrap(), "test interrupt\n").unwrap();
        asm!("cli; hlt", options(nostack));
        loop {}
    }
}

#[no_mangle]
pub fn exception_handler()
{
    unsafe {
        write!(VGA_INSTANCE.as_mut().unwrap(), "CPU Exception !!!!!\n").unwrap();
        loop {}
    }
}

pub fn setup_handlers()
{
    for i in 0..256 {
        unsafe {
            IDT[i].offset_low = ((test_handler as u32) & 0xffff) as u16;
            IDT[i].selector = (1 as u16) << 3;
            IDT[i].zero = 0;
            IDT[i].flags = 0x8e;
            IDT[i].offset_high = (test_handler as u32 >> 16) as u16;
        }
    }
}

pub fn setup()
{
    unsafe {
        setup_handlers();
        IDTR.size = (IDT.len() * core::mem::size_of::<IdtEntry>() - 1) as u16; // TODO why remove 1
        IDTR.offset = &IDT as *const _ as u32;

        asm!(
            "lidt [{0}]",
            in(reg) &IDTR,
            options(nostack, preserves_flags)
        );
        write!(VGA_INSTANCE.as_mut().unwrap(), "IDT pointer : {:x}, size : {:x}\n", &IDTR as *const _ as u32, IDT.len() * core::mem::size_of::<IdtEntry>() - 1).unwrap();
        // write!(VGA_INSTANCE.as_mut().unwrap(), "IDT setup done\n");
    }
}
