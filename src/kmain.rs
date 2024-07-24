#![no_main]
#![no_std]

#![reexport_test_harness_main = "test_main"]
#![feature(custom_test_frameworks)]
#![test_runner(crate::test_runner)]

#![feature(core_intrinsics)]
#![feature(asm_const)]

#[macro_use]
extern crate alloc;

use core::panic::PanicInfo;

mod klib;
mod driver;
mod arch;
mod memory;
mod utils;
mod proc;
mod fs;

// include architecure specific code
pub use arch::*;

use crate::proc::schedule;


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

pub fn spawn_proc_0() {
    let mut val = 0;
    loop {
        klog!("Proc 0 ---- {}", val);
        for _i in 0..1000000 {
        }
        val += 1;
    }
}
pub fn spawn_proc_1() {
    let mut val = 0;
    loop {
        klog!("Proc 1 ---- {}", val);
        for _i in 0..1000000 {
        }
        val += 1;
    }
}


pub fn kmain() -> !
{
    #[cfg(test)]
    klog!("Hello from kmain");
    #[cfg(test)]
    test_main();

    fs::vfs::init();
    
//     schedule::init();
//     schedule::new_kernel_thread(spawn_proc_0);
//     schedule::new_kernel_thread(spawn_proc_1);
//     enable_interrupts();
    loop {
    }
}

#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    klog!("Ceci est une panique \n"); // TODO log macro
    // print panic 
    klog!("{}\n", _info); // TODO log macro


    #[cfg(test)]
    test_main();

    arch::disable_interrupts();

    loop {}
}
