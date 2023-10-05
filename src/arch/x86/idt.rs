use core::arch::asm;
use crate::klog;

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

#[inline(always)]
fn ack_irq()
{
    unsafe {
        // check for slave
    }
}

#[no_mangle]
pub extern "C" fn generic_handler(interrupt_code: u32)
{
    unsafe {
        // write!(VGA_INSTANCE.as_mut().unwrap(), "Interrupt code : {:x}\n", interrupt_code).unwrap();
        klog!("Fucking let's goooo");
        // ack_irq();
        // asm!("push eax",
        //      "mov al, 0x20",
        //      "out 0x20, al",
        //      "pop eax",
        //      options(nostack, preserves_flags));
    }
}

#[no_mangle]
pub fn exception_handler()
{
    klog!("CPU Exception !!!!!");
}

#[no_mangle]
pub fn keystroke_handler(data: u32)
{
    klog!("Keystroke code : {:x}", data);
}

extern "C" {
    fn interrupt_wrapper();
}

pub fn setup_handlers()
{
    for i in 0..0x20 {
        unsafe {
            IDT[i].offset_low = ((exception_handler as u32) & 0xffff) as u16;
            IDT[i].selector = (1 as u16) << 3;
            IDT[i].zero = 0;
            IDT[i].flags = 0x8e;
            IDT[i].offset_high = (exception_handler as u32 >> 16) as u16;
        }
    }
    for i in 0x20..256 {
        unsafe {
            IDT[i].offset_low = ((interrupt_wrapper as u32) & 0xffff) as u16;
            IDT[i].selector = (1 as u16) << 3;
            IDT[i].zero = 0;
            IDT[i].flags = 0x8e;
            IDT[i].offset_high = (interrupt_wrapper as u32 >> 16) as u16;
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
        klog!("IDT pointer : {:x}, size : {:x}", &IDTR as *const _ as u32, IDT.len() * core::mem::size_of::<IdtEntry>() - 1);
    }
}
