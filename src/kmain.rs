#![no_main]
#![no_std]

#![reexport_test_harness_main = "test_main"]
#![feature(custom_test_frameworks)]
#![test_runner(crate::test_runner)]

#![feature(core_intrinsics)]

use core::panic::PanicInfo;
extern crate alloc;

mod klib;
mod driver;
mod arch;
mod memory;
mod utils;

// include architecure specific code
pub use arch::*;


#[cfg(test)]
pub fn test_runner(tests: &[&dyn Fn()]) {
    crate::klog!("Running {} tests", tests.len());
    for test in tests {
        test();
    }
}

#[test_case]
fn trivial_assertion() {
    kprint!("trivial assertion... ");
    assert_eq!(1, 1);
    klog!("[ok]");
}


pub fn kmain() -> !
{
    #[cfg(test)]
    klog!("Hello from kmain");
    #[cfg(test)]
    test_main();
    loop {}
}

#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    klog!("Ceci est une panique \n"); // TODO log macro
    // print panic 
    klog!("{}\n", _info); // TODO log macro


    #[cfg(test)]
    test_main();
    loop {}
}
