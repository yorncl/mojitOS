// TODO maybe move
pub mod input;

pub mod vga;
pub mod kbd;
pub mod timer;
pub mod pci_ide;
pub mod pci;

pub mod serial;

pub trait DriverInterface {
    fn init() -> Result<(),()>;
}
