use core::fmt::Debug;

mod four_wire;
mod four_wire_ref;
mod three_wire;

pub use self::four_wire::FourWire;
pub use self::four_wire::FourWireError;
pub use self::four_wire_ref::FourWireRef;
pub use self::four_wire_ref::OutputPinRef;
pub use self::four_wire_ref::SpiRef;
pub use self::three_wire::ThreeWire;
pub use self::three_wire::ThreeWireError;

pub trait Bus {
    type Error: Debug;

    fn read_frame(&mut self, block: u8, address: u16, data: &mut [u8]) -> Result<(), Self::Error>;

    fn write_frame(&mut self, block: u8, address: u16, data: &[u8]) -> Result<(), Self::Error>;
}
