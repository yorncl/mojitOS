use core::arch::asm;

use core::marker::PhantomData;
#[derive(Copy, Clone)]
pub struct Pio<Op> {
    port: u16,
    _value: PhantomData<Op>,
}

// Small port type with read/write interface
// Concept taken from the redox syscall crate

pub trait PortIO {
    type Op;

    fn read(&self) -> Self::Op;
    fn write(&self, op: Self::Op);
}

impl<T> Pio<T> {
    pub fn new(port: u16) -> Self {
        Pio {
            port,
            _value: PhantomData,
        }
    }
}

impl PortIO for Pio<u8> {
    type Op = u8;

    #[inline]
    fn read(&self) -> Self::Op {
        inb(self.port)
    }

    #[inline]
    fn write(&self, op: Self::Op) {
        outb(self.port, op);
    }
}

impl PortIO for Pio<u16> {
    type Op = u16;

    #[inline]
    fn read(&self) -> Self::Op {
        inw(self.port)
    }

    #[inline]
    fn write(&self, op: Self::Op) {
        outw(self.port, op);
    }
}

impl PortIO for Pio<u32> {
    type Op = u32;

    #[inline]
    fn read(&self) -> Self::Op {
        inl(self.port)
    }

    #[inline]
    fn write(&self, op: Self::Op) {
        outl(self.port, op);
    }
}

// Generic ports on x86
pub mod port {
    // PS2 ports
    pub const PS2DATA: u16 = 0x60;
    // Read to get status, write to send command
    pub const PS2CONTROL: u16 = 0x24;

    // PIC
    pub const PICMASTERCOMMAND: u16 = 0x20;
    pub const PICMASTERDATA: u16 = 0x21;
    pub const PICSLAVECOMMAND: u16 = 0xA0;
    pub const PICSLAVEDATA: u16 = 0xA1;

    // PIT
    pub const PITCHAN0: u16 = 0x40;
    pub const PITCHAN1: u16 = 0x41;
    pub const PITCHAN2: u16 = 0x42;
    pub const PITCONTROL: u16 = 0x43;
    // bit 0
    pub const PITGATE: u16 = 0x61;

    // PCI
    pub const PCICONFIG_ADDRESS: u16 = 0xCF8;
    pub const PCICONFIG_DATA: u16 = 0xCFC;
}

// All the port writing/reading function

// TODO force inline ?
pub fn outb(port: u16, byte: u8) {
    unsafe {
        asm!(
        "out dx, al",
        in("dx") port,
        in("al") byte
        );
    }
}

pub fn inb(port: u16) -> u8 {
    let byte: u8;
    unsafe {
        asm!(
        "in al, dx",
        in("dx") port,
        out("al") byte
        );
    }
    byte
}

pub fn outw(port: u16, word: u16) {
    unsafe {
        asm!(
        "out dx, ax",
        in("dx") port,
        in("ax") word
        );
    }
}

pub fn inw(port: u16) -> u16 {
    let word: u16;
    unsafe {
        asm!(
        "in ax, dx",
        in("dx") port,
        out("ax") word
        );
    }
    word
}

pub fn outl(port: u16, long: u32) {
    unsafe {
        asm!(
        "out dx, eax",
        in("dx") port,
        in("eax") long
        );
    }
}

pub fn inl(port: u16) -> u32 {
    let long: u32;
    unsafe {
        asm!(
        "in eax, dx",
        in("dx") port,
        out("eax") long
        );
    }
    long
}

// TODO wtf ? I don't remember why I did that
pub fn wait() {
    unsafe { asm!("mov al, 0", "mov dx, 0x80", "out dx, al") }
}
