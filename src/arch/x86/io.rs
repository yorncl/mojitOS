use core::arch::asm;

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

    let byte : u8;
    unsafe {
        asm!(
            "in al, dx",
            in("dx") port,
            out("al") byte
            );
    }
    byte
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
