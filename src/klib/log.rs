
#[macro_export]
macro_rules! print { // TODO maybe rename to klog
    ($($arg:tt)*) => ($crate::driver::vga::_print(format_args!($($arg)*)));
}

#[macro_export]
macro_rules! klog { // TODO maybe rename to klogn
    () => ($crate::print!("\n"));
    ($($arg:tt)*) => ($crate::print!("{}\n", format_args!($($arg)*)));
}

