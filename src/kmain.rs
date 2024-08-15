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
mod error;
mod fs;
mod klib;
mod memory;
mod proc;
mod utils;

// include architecure specific code
pub use arch::*;

use crate::proc::schedule::{self, schedule};

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
    let mut _val = 0;
    loop {
        dbg!("Proc 0 ---- {}", _val);
        for _i in 0..100000 {}
        _val += 1;
    }
}
#[allow(dead_code)]
pub fn spawn_proc_1() {
    let mut _val = 0;
    loop {
        dbg!("Proc 1 ---- {}", _val);
        for _i in 0..100000 {}
        _val += 1;
    }
}

use crate::fs::block;

/// The main loop of the kernel
pub fn kmain() -> ! {
    // PS/2 keyboard driver with ISA interrupts
    driver::kbd::init().unwrap();

    // Enumerating PCI bus
    driver::pci::init();
    klog!("Enumerating PCI devices");
    for pci_dev in driver::pci::get_devices() {
        match pci_dev.kind {
            // ATA/IDE
            driver::pci::PCIType::IDE => {
                // probe the controller
                if let Some(drv) = driver::pci_ide::IDEController::probe_controller(pci_dev) {
                    // Register block devices from detected ATA disks if any
                    for bus_lock in drv.buses {
                        let bus = bus_lock.read().unwrap();
                        let disks = bus.disks.read().unwrap();
                        for d in disks.iter() {
                            klog!("  IDE drive");
                            block::register_device(d.clone());
                        }
                    }
                }
            }
            _ => {}
        }
    }

    // Will panic if no block have been registered
    klog!("Initializing filesystems");
    block::init_fs_from_devices();

    // TODO id system for partition, kernel boot arg, something like ata1hd0part1
    // let part = 0x0100;

    let fss = fs::vfs::get_filesystems();
    if fss.len() > 1 {
        panic!("Don't know how to choose vfs root!");
    }
    // TODO ugly
    fs::vfs::register_mount("/", fss[0].clone());

    let file = fs::vfs::open("/home/yrn/hello-world");

    // After the first scheduler tick, the execution context will not come back to this loop
    schedule::init();
    // schedule::new_kernel_thread(spawn_proc_0);
    // schedule::new_kernel_thread(spawn_proc_1);
    arch::enable_interrupts();
    loop {}
}

#[panic_handler]
/// The panic handler for the kernel
fn panic(_info: &PanicInfo) -> ! {
    arch::disable_interrupts();
    // TODO could it dead lock ?
    dbg!("Kernel panic!\n");
    klog!("Kernel panic!\n");
    dbg!("{}\n", _info); // TODO log macro
    klog!("{}\n", _info); // TODO log macro
    loop {}
}
