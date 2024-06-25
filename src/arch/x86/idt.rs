use core::arch::asm;
use crate::klog;
use super::paging::page_fault_handler;

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
pub fn ack_irq()
{
}

#[no_mangle]
pub extern "C" fn generic_handler(interrupt_code: u32)
{
        klog!("Fucking let's goooo {}", interrupt_code);
        // ack_irq();
        // asm!("push eax",
        //      "mov al, 0x20",
        //      "out 0x20, al",
        //      "pop eax",
        //      options(nostack, preserves_flags));
}

#[no_mangle]
pub fn exception_handler(code:u32)
{
    klog!("CPU Exception !!!!!");
    loop{}
}

#[no_mangle]
pub fn keystroke_handler(data: u32)
{
    klog!("Keystroke code : {:x}", data);
}

extern "C" {
    fn interrupt_wrapper(data: u32);
}

fn set_exception(i: usize, handler: u32, selector: u16, flags: u8)
{
    unsafe {
        IDT[i].offset_low = ((handler as u32) & 0xffff) as u16;
        IDT[i].selector = selector;
        IDT[i].zero = 0;
        IDT[i].flags = flags;
        IDT[i].offset_high = (exception_handler as u32 >> 16) as u16;
    }
}

// TODO hangler type is ugly
fn set_interrupt(i: usize, handler: unsafe extern fn (u32) -> (), selector: u16, flags: u8)
{
    unsafe {
        IDT[i].offset_low = ((handler as u32) & 0xffff) as u16;
        IDT[i].selector = selector;
        IDT[i].zero = 0;
        IDT[i].flags = flags;
        IDT[i].offset_high = (exception_handler as u32 >> 16) as u16;
    }
}

// TODO spurious vector interrupts
fn setup_handlers()
{
    for i in 0..0x20 {
        set_exception(i, exception_handler as u32, 1 << 3, 0x8e);
    }
    set_exception(0xe, page_fault_handler as u32, 1 << 3, 0x8e);
    for i in 0x20..256 {
        set_interrupt(i, interrupt_wrapper, 1 << 3, 0x8e);
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
