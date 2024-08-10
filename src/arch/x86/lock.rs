use core::arch::asm;
// use core::mem;
use core::ptr::addr_of;

// static mut THELOCK: u32 = 0;

pub struct RawSpinLock {
    state: usize
}

/// Raw spin lock
impl RawSpinLock {

    /// Creates a new RawSpinLock in an available state
    pub const fn new() -> RawSpinLock {
        RawSpinLock {state: 0}
    }

    /// This function will attempt to exchange the desired value until it succeeds
    /// when the swap reveals that the two values are different, it has been successful
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

    /// spin lock
    pub fn lock(&self) {
            while self.exchange(1) {
            }
    }

    /// spin lock release
    pub fn release(&self) {
            self.exchange(0);
    }
}


