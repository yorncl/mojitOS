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
static HELLO: &[u8] = b"1\n2\n3\n4\n5\n6\n7\n8\n9\n10\n11\n12\n13\n14\n15\n16\n17\n18\n19\n20\n21\n22\n23\n24\n25\n26\n27\n28\n29\n30\n31\n32\n";

#[no_mangle]
pub extern "C" fn _start() -> ! {

    let mut vga_drv = vga::VGA::new();
    vga_drv.puts(HELLO);
    loop {}
}
