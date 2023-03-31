
// A simple vga text driver
use crate::klib::mem;
use volatile::Volatile;

const HEIGHT: usize = 25;
const WIDTH: usize = 80;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(C)]
struct VgaChar
{
    c : u8,
    col : u8 // TODO implement color
}

#[repr(transparent)]
struct Buffer
{
   data: [[Volatile<VgaChar>; WIDTH]; HEIGHT]
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

    pub fn clear(&mut self)
    {
        mem::memset(self.buffer.data.as_ptr() as *mut u8, 0, HEIGHT * WIDTH);
        self.x = 0;
        self.y = 0;
    }
    
    pub fn new_line(&mut self)
    {
        self.y += 1;
        self.x = 0;
        if self.y == HEIGHT {
            mem::memcpy(
                self.buffer.data.as_ptr() as *mut u8,
                self.buffer.data[1].as_ptr() as *const u8,
                WIDTH * 2 * (HEIGHT -1));
            mem::memset(self.buffer.data[HEIGHT - 1].as_ptr() as *mut u8, 0, WIDTH * 2);
            self.y = HEIGHT - 1;
        }
    }

    pub fn puts(&mut self, s: &[u8]) {
        let mut i: usize = 0;

        while i < s.len() {
            let b: u8 = s[i];
            match b 
            {
                0x20..=0x7e => {
                    if self.x == WIDTH {
                        self.new_line();
                    }
                    self.buffer.data[self.y][self.x].write(VgaChar { c: b, col: 15 });
                    self.x += 1;
                },
                b'\n' => {
                    self.new_line();
                } 
                _ => {}
            }
            i += 1;
         }
    }

}
