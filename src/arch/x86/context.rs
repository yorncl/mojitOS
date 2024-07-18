use crate::{klog, x86::apic};
use core::{arch::asm, mem::{size_of, self}, mem::offset_of, usize};

use alloc::vec::Vec;

#[repr(C)]
#[derive(Default, Clone)]
pub struct Context {
    // General purpose
    pub eax: u32,
    pub ebx: u32,
    pub ecx: u32,
    pub edx: u32,
    pub esi: u32,
    pub edi: u32,
    // Stack
    pub ebp: u32,
    pub esp: u32,
    // Instruction pointer
    pub eip: u32,
    // pub cs: u32,
    pub eflags: u32,

    // Segment registers
    // es: u32,
    // cs: u32,
    // ss: u32,
    // ds: u32,
    // fs: u32,
    // gs: u32,

    pub stack: Vec<u8>
    // TODO FPU, SSE et tutti quanti
}

impl Context {

    pub fn init_stack(&mut self) {
        self.stack = vec![0 as u8; 4096];
        // klog!("New stack allocated at {:x}", self.stack.as_ptr() as usize);
        self.ebp = self.stack.as_ptr() as u32 + 4096;
        self.esp = self.ebp;
    }

    pub fn push(&mut self, value: u32) {
        unsafe {
            self.esp -= mem::size_of::<u32>() as u32;
            *(self.esp as *mut u32) = value;
            // klog!("NEW ESP {:x}, stored value {:x}", self.esp, *((self.esp + 4) as *mut u32) );
        }
    }
}

// TODO think of unlock context in the future, cf Redox
// EAX, ECX, EDX saved by caller
#[no_mangle]
extern "cdecl" fn switch_inner(prev: &mut Context, next: &mut Context) {
    unsafe {
        // TODO eflags
        asm!(
            concat!("

                pushfd
                mov eax, [esp]
                mov [ecx + {off_eflags}], eax
                mov eax, [edx + {off_eflags}]
                mov [esp], eax
                popfd

                mov [ecx + {off_ebx}], ebx
                mov [ecx + {off_esi}], esi
                mov [ecx + {off_edi}], edi
                mov ebx, [edx + {off_ebx}]
                mov esi, [edx + {off_ebx}]
                mov edi, [edx + {off_ebx}]
                
                mov [ecx + {off_esp}], esp
                mov [ecx + {off_ebp}], ebp
                mov esp, [edx + {off_esp}]
                mov ebp, [edx + {off_ebp}]
            "
            ),
            in("ecx") prev,
            in("edx") next,
            off_ebx = const(offset_of!(Context, ebx)),
            off_esi= const(offset_of!(Context, esi)),
            off_edi = const(offset_of!(Context, edi)),
            off_esp = const(offset_of!(Context, esp)),
            off_ebp = const(offset_of!(Context, ebp)),
            off_eflags = const(offset_of!(Context, eflags)),
            options(nostack)
        );
    }
}


#[no_mangle]
pub fn switch(prev: &mut Context, next: &mut Context) {
    // klog!("PREV EBP {:x} ESP {:x}", prev.ebp, prev.esp);
    // klog!("NEXT EBP {:x} ESP {:x}", next.ebp, next.esp);
    switch_inner(prev, next);
}
