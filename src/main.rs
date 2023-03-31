#![no_main]
#![no_std]

use core::panic::PanicInfo;

mod klib { pub mod mem; }
mod driver { pub mod vga; }
use driver::vga;


/// This function is called on panic.
#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    loop {}
}

// static HELLO: &[u8] = b"Super les gars c'est cool";
static HELLO: &[u8] = b"Super les gars c'est cool\non est la sisi tavu";

#[no_mangle]
pub extern "C" fn _start() -> ! {

    let mut vga_drv = vga::VGA::new();
    vga_drv.puts(HELLO);
    loop {}
}
