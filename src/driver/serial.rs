/// This driver will interface with port COM1
/// the dbg macro will print to that port directly, rather than to the port 

use core::fmt;
use core::fmt::Write;
use crate::arch::io::Pio;
use crate::io::PortIO;
use crate::klib::lock::RwLock;

#[macro_export]
macro_rules! dbg_print {
    ($($arg:tt)*) => ($crate::driver::serial::_print(format_args!($($arg)*)));
}

#[macro_export]
macro_rules! dbg {
        () => {
            #[cfg(feature = "debug_serial")]
            ($crate::dbg_print!("\n"))
        };
        ($($arg:tt)*) => {
            #[cfg(feature = "debug_serial")]
            ($crate::dbg_print!("{}\n", format_args!($($arg)*)))
        };
}

struct SerialDriver {
     com1: Pio<u8>
}

static mut COM1: RwLock<SerialDriver> = RwLock::new(SerialDriver { com1: Pio::new(0x3f8) });

pub fn _print(args: fmt::Arguments) {
    unsafe {
        let mut driver = COM1.write().unwrap();
        driver.write_fmt(args).unwrap();
    }
}

impl SerialDriver {
    pub fn puts(&mut self, bytes: &[u8]) {
        for b in bytes {
            self.com1.write(*b);
        }
    }
}

impl Write for SerialDriver {
    fn write_str(&mut self, s: &str) -> core::fmt::Result {
        self.puts(&s.as_bytes());
        Ok(())
    }
}
