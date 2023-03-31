
// A simple vga text driver
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(C)]
struct VgaChar
{
    c : u8,
    col : u8 // TODO implement color
}

const HEIGHT: usize = 25;
const WIDTH: usize = 80;

#[repr(transparent)]
struct Buffer
{
   data: [[VgaChar; WIDTH]; HEIGHT]
}

pub struct VGA {
    buffer: &'static mut Buffer,
    x: usize,
    y: usize,
}

impl VGA {
    pub fn new() -> VGA {
        VGA {
            buffer: unsafe { &mut *(0xb8000 as *mut Buffer)}, 
            x: 0,
            y: 0,
        }
    }

    pub fn puts(&mut self, s: &[u8]) {
    let mut i: usize = 0;

    while i < s.len() {
        let b: u8 = s[i];
        match b 
        {
            0x20..=0x7e => {
                self.buffer.data[self.y][self.x] = VgaChar { c: b, col: 15 };
                self.x += 1;
            },
            b'\n' => {
                self.y += 1;
                self.x = 0;
            } 
            _ => {}
        }
        // checking for lines and column overflow
        if self.x == WIDTH {
            self.x = 0;
            self.y += 1;
        }
        if self.y == HEIGHT {
            return;
        }
        i += 1;
    }
}

}
