use core::arch::asm;
use core::mem;
use core::ptr::addr_of;

static mut THELOCK: u32 = 0;


pub const TAKEN: u32 = 1;
pub const FREE: u32 = 0;

#[no_mangle]
pub fn swap_atomic (order: u32) -> bool {
    let mut value = order;
    unsafe {
        asm!(
            "xchg {input},[{lock}]",
            lock = in(reg) addr_of!(THELOCK) as usize,
            input = inout(reg) value
        );
    }
    if value == order {
        return false
    }
    true
}

pub fn spin_lock() {
    unsafe {
        while swap_atomic(TAKEN) == false {
        }
    }
}

pub fn spin_release() {
    unsafe {
        swap_atomic(FREE);
    }
}

