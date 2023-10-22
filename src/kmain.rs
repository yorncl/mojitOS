#![no_main]
#![no_std]

use core::panic::PanicInfo;

mod klib;
mod driver;
mod arch;

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
