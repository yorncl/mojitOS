use crate::arch::io;

use super::input;
use crate::arch::irq;
use crate::arch::io::Port;


#[repr(u8)]
enum Command {
    ReadConf = 0x20,
}

#[inline(always)]
fn read_data() -> u8 {
    io::inb(Port::PS2Data)
}

fn read_conf_byte() -> u8 {
    io::outb(Port::PS2Control, Command::ReadConf as u8);
    io::inb(Port::PS2Data)
}

fn int_handler() -> Result<(),()> {
    let event = read_data();
    input::push_event(input::InputEvent::Keyboard(event as u32));
    Ok(())
}

pub fn init() -> Result<(),()> {
    let conf = read_conf_byte();
    if conf & (1 << 6) != 0 {
        // panic!("PS2 translation enabled");
    }
    if irq::request_irq(42, int_handler).is_err() {
        panic!("Could not init keyboard driver!");
    }
    Ok(())
}
