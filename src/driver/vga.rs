
// A simple vga text driver
use crate::klib::mem;
use core::fmt;
use core::fmt::Write;
use core::ffi::c_void;

const HEIGHT: usize = 25;
const WIDTH: usize = 80;
pub static mut VGA_INSTANCE: Option<VGA> = None;

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
   data: [[VgaChar; WIDTH]; HEIGHT] // TODO implement volatile to avoid compiler optimizations
}

pub struct VGA {
    buffer: &'static mut Buffer,
    x: usize,
    y: usize,
}

#[no_mangle]
pub fn io_init()
{
    unsafe {
        VGA_INSTANCE = Some(VGA::new());
        VGA_INSTANCE.as_mut().unwrap().clear();
    }
}

#[doc(hidden)]
pub fn _print(args: fmt::Arguments) {
    unsafe {
        VGA_INSTANCE.as_mut().unwrap().write_fmt(args).unwrap();
    }
}

impl VGA {
    pub fn new() -> VGA {
        VGA {
            // buffer: unsafe { &mut *(0xb8000 as *mut Buffer)}, 
            buffer: unsafe { &mut *(0xC00b8000 as *mut Buffer)}, 
            x: 0,
            y: 0,
        }
    }

    pub fn clear(&mut self)
    {
        mem::memset(self.buffer.data.as_ptr() as *mut c_void, 0, HEIGHT * WIDTH);
        self.x = 0;
        self.y = 0;
    }
    
    pub fn new_line(&mut self)
    {
        self.y += 1;
        self.x = 0;
        if self.y == HEIGHT {
            mem::memcpy(
                self.buffer.data.as_ptr() as *mut c_void,
                self.buffer.data[1].as_ptr() as *const c_void,
                WIDTH * 2 * (HEIGHT -1));
            mem::memset(self.buffer.data[HEIGHT - 1].as_ptr() as *mut c_void, 0, WIDTH * 2);
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
                    self.buffer.data[self.y][self.x] = VgaChar { c: b, col: 15 };
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


// TODO probably temp
impl Write for VGA
{
    fn write_str(&mut self, s:&str) -> core::fmt::Result
    {
        self.puts(&s.as_bytes());
        Ok(())
    }

}
