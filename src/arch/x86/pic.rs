use crate::arch::x86::io;
use crate::arch::x86::io::port;


#[allow(dead_code)]
#[repr(u8)]
enum ICW1 {
    ICW4 = 0x01,        // Indicates that ICW4 will be present
    SINGLE = 0x02,      // Single (cascade) mode
    INTERVAL4 = 0x04,   // Call address interval 4 (8)
    LEVEL = 0x08,       // Level triggered (edge) mode
    INIT = 0x10,        // Initialization - required!
    EOI = 0x20,         // End of interrupt
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
    // let master_mask = io::inb(port::PICMASTERDATA);
    // let slave_mask = io::inb(port::PICSLAVEDATA);

    // klog!("PIC MASKS {:b} {:b}", master_mask, slave_mask);

    io::outb(port::PICMASTERCOMMAND, ICW1::INIT as u8 | ICW1::ICW4 as u8); // PIC reset
    io::wait();
    io::outb(port::PICSLAVECOMMAND, ICW1::INIT as u8 | ICW1::ICW4 as u8); // PIC resetS
    io::wait();

    // remap 
    io::outb(port::PICMASTERDATA, offset1 as u8);
    io::wait();
    io::outb(port::PICSLAVEDATA, offset2 as u8);
    io::wait();
    io::outb(port::PICMASTERDATA, 4);
    io::wait();
    io::outb(port::PICSLAVEDATA, 2);
    io::wait();


    io::outb(port::PICMASTERDATA, ICW4::_8086 as u8);
    io::wait();
    io::outb(port::PICSLAVEDATA, ICW4::_8086 as u8);
    io::wait();

    // rewrite saved masks 
    io::outb(port::PICMASTERDATA, 1); // TODO this "as u16" is ugly, can we find a
                                                       // better way
    io::outb(port::PICSLAVEDATA, 0);
    io::wait();
}

pub fn eoi(interrupt_code: u32) {
    if interrupt_code >= 8 {
        io::outb(port::PICSLAVECOMMAND, ICW1::EOI as u8);
    }
    io::outb(port::PICMASTERCOMMAND, ICW1::EOI as u8);
}

pub fn setup()
{
    pic_remap(0x20 as i8, 0x28 as i8); // the first 32 interrupts are reserved for the CPU exceptions
}

pub fn disable() {
    setup();
    io::outb(port::PICMASTERDATA, 0xFF);
    io::outb(port::PICSLAVEDATA, 0xFF);
    // TODO is all that waiting necessary ?
    io::wait();
}
