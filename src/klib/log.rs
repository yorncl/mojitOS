
#[macro_export]
macro_rules! print {
    ($($arg:tt)*) => ($crate::driver::vga::_print(format_args!($($arg)*)));
}

#[macro_export]
macro_rules! klog {
    () => ($crate::print!("\n"));
    ($($arg:tt)*) => ($crate::print!("{}\n", format_args!($($arg)*)));
}

