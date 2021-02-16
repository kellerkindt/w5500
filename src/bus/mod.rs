use core::fmt::Debug;

mod four_wire;
mod three_wire;

pub use self::four_wire::FourWire;
pub use self::three_wire::ThreeWire;

pub trait Bus {
    type Error: Debug;

    fn read_frame(&mut self, block: u8, address: u16, data: &mut [u8]) -> Result<(), Self::Error>;

    fn write_frame(&mut self, block: u8, address: u16, data: &[u8]) -> Result<(), Self::Error>;
}
