use core::arch::asm;
// use core::mem;
use core::ptr::addr_of;

// static mut THELOCK: u32 = 0;

pub struct RawSpinLock {
    state: usize
}

impl RawSpinLock {

    pub const fn new() -> RawSpinLock {
        RawSpinLock {state: 0}
    }

    fn exchange(&self, order: usize) -> bool {
        let mut value = order;
        unsafe {
            asm!(
                "lock xchg {input},[{lock}]",
                lock = in(reg) addr_of!(self.state) as usize,
                input = inout(reg) value
            );
        }
        value == order
    }

    pub fn lock(&self) {
            while self.exchange(1) {
            }
    }

    pub fn release(&self) {
            self.exchange(0);
    }
}


