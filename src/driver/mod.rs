// TODO maybe move
pub mod input;

pub mod vga;
pub mod kbd;

pub trait DriverInterface {
    fn init() -> Result<(),()>;
}
