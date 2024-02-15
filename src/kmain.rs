#![no_main]
#![no_std]

use core::panic::PanicInfo;
extern crate alloc;

mod klib;
mod driver;
mod arch;
mod memory;
mod utils;

// include architecure specific code
pub use arch::*;

pub fn kmain() -> !
{
    klog!("Hello from kmain");
    loop {}
}

#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    klog!("Ceci est une panique \n"); // TODO log macro
    // print panic 
    klog!("{}\n", _info); // TODO log macro
    loop {}
}
