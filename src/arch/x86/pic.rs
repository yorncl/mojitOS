use crate::arch::x86::io;

#[allow(dead_code)]


#[derive(Debug)]
#[repr(u16)]
enum PicPort {
    MasterCommand = 0x20,
    MasterData = 0x21,
    SlaveCommand = 0xA0,
    SlaveData = 0xA1,
}

#[allow(dead_code)]
#[repr(u8)]
enum ICW1 {
    ICW4 = 0x01,        // Indicates that ICW4 will be present
    SINGLE = 0x02,      // Single (cascade) mode
    INTERVAL4 = 0x04,   // Call address interval 4 (8)
    LEVEL = 0x08,       // Level triggered (edge) mode
    INIT = 0x10,        // Initialization - required!
}
// TODO implem into/from trait to avoid casting ?

#[allow(dead_code)]
#[repr(u8)]
enum ICW4 {
    _8086 = 0x01,       // 8086/88 (MCS-80/85) mode
    AUTO = 0x02,        // Auto (normal) EOI
    BUFSLAVE = 0x08,   // Buffered mode/slave
    BUFMASTER = 0x0C,  // Buffered mode/master
    SFNM = 0x10,        // Special fully nested (not)
}

fn pic_remap(offset1: u8, offset2: u8)
{
    // save masks
    let master_mask = io::inb(PicPort::MasterData as u16);
    let slave_mask = io::inb(PicPort::SlaveData as u16);

    io::outb(PicPort::MasterCommand as u16, ICW1::INIT as u8 | ICW1::ICW4 as u8); // PIC reset
    io::wait();

    // remap 
    io::outb(PicPort::MasterData as u16, offset1);
    io::wait();
    io::outb(PicPort::SlaveData as u16, offset2);
    io::wait();
    io::outb(PicPort::MasterData as u16, 4);
    io::wait();
    io::outb(PicPort::SlaveData as u16, 2);
    io::wait();

    // rewrite saved masks 
    io::outb(PicPort::MasterData as u16, master_mask);
    io::wait(); // TODO necessary ?
    io::outb(PicPort::SlaveData as u16, slave_mask);
}

pub fn setup()
{
    pic_remap(0x20, 0x28); // the first 32 interrupts are reserved for the CPU exceptions
}
