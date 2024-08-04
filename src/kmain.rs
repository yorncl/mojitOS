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

mod arch;
mod driver;
mod fs;
mod klib;
mod memory;
mod proc;
mod utils;

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
        for _i in 0..1000000 {}
        val += 1;
    }
}
pub fn spawn_proc_1() {
    let mut val = 0;
    loop {
        klog!("Proc 1 ---- {}", val);
        for _i in 0..1000000 {}
        val += 1;
    }
}

use alloc::boxed::Box;
use alloc::vec::Vec;
use fs::block;

pub fn kmain() -> ! {
    #[cfg(test)]
    klog!("Hello from kmain");
    #[cfg(test)]
    test_main();

    driver::pci::init();
    // TODO remove
    arch::enable_interrupts();

    for pci_dev in driver::pci::get_devices() {
        match pci_dev.kind {
            // ATA/IDE
            driver::pci::PCIType::IDE => {
                // probe the controller
                if let Some(drv) = driver::pci_ide::IDEController::probe_controller(pci_dev) {
                    // Register block devices from detected ATA disks if any
                    for b in drv.buses {
                        for disk in b.borrow_mut().disks.iter() {
                            klog!("Some DRIVER IDE");
                            block::register_device(disk.clone());
                        }
                    }
                }
            }
            _ => {}
        }
    }

    // Will panic if no block have been registered
    block::init_fs_from_devices();

    fs::fs::moun

    //     schedule::init();
    //     schedule::new_kernel_thread(spawn_proc_0);
    //     schedule::new_kernel_thread(spawn_proc_1);
    //     enable_interrupts();
    loop {}
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
