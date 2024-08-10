use crate::arch::io;
use crate::klog;

use super::input;
use crate::arch::irq;
use crate::arch::io::port;


#[repr(u8)]
enum Command {
    ReadConf = 0x20,
}

#[inline(always)]
fn read_data() -> u8 {
    io::inb(port::PS2DATA)
}

fn read_conf_byte() -> u8 {
    io::outb(port::PS2CONTROL, Command::ReadConf as u8);
    io::inb(port::PS2DATA)
}

fn int_handler() -> Result<(),()> {
    let event = read_data();
    input::push_event(input::InputEvent::Keyboard(event as u32));
    Ok(())
}

/// Init the PS2/Keyboard driver
pub fn init() -> Result<(),()> {
    let conf = read_conf_byte();
    if conf & (1 << 6) != 0 {
        // panic!("PS2 translation enabled");
    }
    // TODO remap to sensible number
    if irq::request_irq_top(42, int_handler).is_err() {
        panic!("Could not init keyboard driver!");
    }
    Ok(())
}
