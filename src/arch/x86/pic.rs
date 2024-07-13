use crate::arch::x86::io;
use crate::arch::x86::io::Port;


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

fn pic_remap(offset1: i8, offset2: i8)
{
    // save masks
    // let master_mask = io::inb(Port::PICMasterData);
    let slave_mask = io::inb(Port::PICSlaveData);

    io::outb(Port::PICMasterCommand, ICW1::INIT as u8 | ICW1::ICW4 as u8); // PIC reset
    io::wait();
    io::outb(Port::PICSlaveCommand, ICW1::INIT as u8 | ICW1::ICW4 as u8); // PIC resetS
    io::wait();

    // remap 
    io::outb(Port::PICMasterData, offset1 as u8);
    io::wait();
    io::outb(Port::PICSlaveData, offset2 as u8);
    io::wait();
    io::outb(Port::PICMasterData, 4);
    io::wait();
    io::outb(Port::PICSlaveData, 2);
    io::wait();


    io::outb(Port::PICMasterData, ICW4::_8086 as u8);
    io::wait();
    io::outb(Port::PICSlaveData, ICW4::_8086 as u8);
    io::wait();

    // rewrite saved masks 
    io::outb(Port::PICMasterData, 0xFD); // TODO this "as u16" is ugly, can we find a
                                                       // better way
    io::outb(Port::PICSlaveData, slave_mask);
    io::wait();
}

pub fn setup()
{
    pic_remap(0x20 as i8, 0x28 as i8); // the first 32 interrupts are reserved for the CPU exceptions
}

pub fn disable() {
    setup();
    io::outb(Port::PICMasterData, 0xFF);
    io::outb(Port::PICSlaveData, 0xFF);
    // TODO is all that waiting necessary ?
    io::wait();
}
