use core::arch::asm;
// use core::mem;
use core::ptr::addr_of;

// static mut THELOCK: u32 = 0;

pub struct SpinLock {
    state: usize
}

impl SpinLock {

    pub fn new() -> SpinLock {
        SpinLock {state: 0}
    }

    fn exchange(&mut self, order: usize) -> bool {
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

    pub fn lock(&mut self) {
            while self.exchange(1) {
            }
    }

    pub fn release(&mut self) {
            self.exchange(0);
    }
}


