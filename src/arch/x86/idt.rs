use core::arch::asm;
use crate::{klog, memory::vmm::mapper, irq, x86::apic};
use super::paging::page_fault_handler;
use core::ptr::addr_of;

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

extern "C" {
    fn fill_idt(idt_address: usize);
}

#[no_mangle]
pub extern "C" fn generic_handler(interrupt_code: u32)
{
        klog!("Interrupt Irq={}", interrupt_code);
        // TODO temporary (and not generic enough to handle PIC only)?
        irq::raise_irq(interrupt_code);
        super::apic::end_of_interrupt();
}

#[no_mangle]
pub extern "C" fn fill_idt_entry(i: usize, handler_address: u32)
{
    unsafe {
        IDT[i].offset_low = (handler_address & 0xffff) as u16;
        // index must be shifted by 3 bits because
        IDT[i].selector = 1 << 3;
        IDT[i].zero = 0;
        // 8 is for present, e is for interrupt gate type
        IDT[i].flags = 0x8e;
        IDT[i].offset_high = (handler_address >> 16) as u16;
    }
}

pub fn setup()
{
    unsafe {
        fill_idt(IDT.as_ptr() as usize);

        IDTR.size = (IDT.len() * core::mem::size_of::<IdtEntry>() - 1) as u16; // TODO why remove 1
        IDTR.offset = addr_of!(IDT) as *const _ as u32;

        asm!(
            "lidt [{0}]",
            in(reg) addr_of!(IDTR),
            options(nostack, preserves_flags)
        );
        klog!("IDT pointer : {:x}, size : {:x}", addr_of!(IDTR) as *const _ as u32, IDT.len() * core::mem::size_of::<IdtEntry>() - 1);
    }
}
