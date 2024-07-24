use core::arch::asm;

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

pub fn outb(port: u16, byte:u8)
{
    unsafe {
        asm!(
            "out dx, al",
            in("dx") port,
            in("al") byte
            );
    }
}

pub fn inb(port: u16) -> u8
{

    let byte : u8; unsafe {
        asm!(
            "in al, dx",
            in("dx") port,
            out("al") byte
            );
    }
    byte
}

pub fn outl(port: u16, long:u32)
{
    unsafe {
        asm!(
            "out dx, eax",
            in("dx") port,
            in("eax") long
            );
    }
}

pub fn inl(port: u16) -> u32
{

    let long : u32;
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
pub fn wait()
{
    unsafe {
        asm!(
            "mov al, 0",
            "mov dx, 0x80",
            "out dx, al"
            )
    }
}
