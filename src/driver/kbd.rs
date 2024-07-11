use crate::arch::io;

use super::DriverInterface;
use super::input;
use crate::arch::irq;

#[repr(u16)]
enum Port {
    Data = 0x60,
    // Read to get status, write to send command
    Control = 0x24,
}

#[repr(u8)]
enum Command {
    ReadConf = 0x20,
}

#[inline(always)]
fn read_data() -> u8 {
    io::inb(Port::Data as u16)
}

fn read_conf_byte() -> u8 {
    io::outb(Port::Control as u16, Command::ReadConf as u8);
    io::inb(Port::Data as u16)
}

fn int_handler() -> Result<(),()> {
    let event = read_data();
    input::push_event(input::InputEvent::Keyboard(event as u32));
    Ok(())
}

pub fn init() -> Result<(),()> {
    let conf = read_conf_byte();
    if conf & (1 << 6) != 0 {
        panic!("PS2 translation enabled");
    }
    if irq::request_irq(42, int_handler).is_err() {
        panic!("Could not init keyboard driver!");
    }
    Ok(())
}
