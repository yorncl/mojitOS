use core::arch::asm;

#[repr(u16)]
pub enum Port {
    // PS2 ports
    PS2Data = 0x60,
    // Read to get status, write to send command
    PS2Control = 0x24,

    // PIC
    PICMasterCommand = 0x20,
    PICMasterData = 0x21,
    PICSlaveCommand = 0xA0,
    PICSlaveData = 0xA1,

    // PIT
    PITChan0 = 0x40,
    PITChan1 = 0x41,
    PITChan2 = 0x42,
    PITControl = 0x43,
    // bit 0
    PITGate = 0x61,
    
    // PCI
    PCICONFIG_ADDRESS = 0xCF8,
    PCICONFIG_DATA = 0xCFC,
}

pub fn outb(port: Port, byte:u8)
{
    unsafe {
        asm!(
            "out dx, al",
            in("dx") port as u16,
            in("al") byte
            );
    }
}

pub fn inb(port: Port) -> u8
{

    let byte : u8; unsafe {
        asm!(
            "in al, dx",
            in("dx") port as u16,
            out("al") byte
            );
    }
    byte
}

pub fn outl(port: Port, long:u32)
{
    unsafe {
        asm!(
            "out dx, eax",
            in("dx") port as u16,
            in("eax") long
            );
    }
}

pub fn inl(port: Port) -> u32
{

    let long : u32;
    unsafe {
        asm!(
            "in eax, dx",
            in("dx") port as u16,
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
