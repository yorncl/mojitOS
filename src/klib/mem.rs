// Common memory operations for the kernel
// TODO optimizations

use core::ffi::c_void;

#[no_mangle]
pub extern "C" fn memcpy(dest: *mut c_void, src: *const c_void, n: usize) -> *mut u8 {
    let dest = dest as *mut u8;
    let src = src as *const u8;

    let mut i = 0;
    while i < n {
        unsafe {
            *dest.add(i) = *src.add(i);
        }
        i += 1;
    }
    dest
}

#[no_mangle]
pub extern "C" fn memmove(dest: *mut c_void, src: *const c_void, n: usize) -> *mut u8 {
    let dest = dest as *mut u8;
    let src = src as *const u8;

    let mut i = 0;
    while i < n {
        unsafe {
            *dest.add(i) = *src.add(i);
        }
        i += 1;
    }
    dest
}

#[no_mangle]
pub extern "C" fn memset(dest: *mut c_void, val: u8, n: usize) -> *mut u8 {
    let mut i = 0;
    let dest = dest as *mut u8;

    while i < n {
        unsafe {
            *dest.add(i) = val;
        }
        i += 1;
    }
    dest
}

#[no_mangle]
pub fn memcmp(s1: *const c_void, s2: *const c_void, n: usize) -> i32 {
    let mut i = 0;
    let s1 = s1 as *const u8;
    let s2 = s2 as *const u8;

    while i < n {
        unsafe {
            if s1.add(i) != s2.add(i) {
                return *s1.add(i) as i32 - *s2.add(i) as i32;
            }
        }
        i += 1;
    }
    0
}


#[no_mangle]
pub fn strlen(s1: *const u8) -> usize { // todo reimplement, doubt on perfomance
    let mut len : usize = 0;

    while unsafe { *s1.add(len) } != 0 {
        len += 1;
    }
    len
}


