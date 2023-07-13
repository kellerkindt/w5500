use core::fmt::Debug;

mod four_wire;
mod four_wire_ref;
mod three_wire;

use crate::register::{
    self,
    common::{PhyConfig, RetryCount, RetryTime, MODE, VERSION},
};

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

pub struct BusRef<'a, B: Bus>(pub &'a mut B);

impl<B: Bus> Bus for BusRef<'_, B> {
    type Error = B::Error;

    #[inline]
    fn read_frame(&mut self, block: u8, address: u16, data: &mut [u8]) -> Result<(), Self::Error> {
        self.0.read_frame(block, address, data)
    }

    #[inline]
    fn write_frame(&mut self, block: u8, address: u16, data: &[u8]) -> Result<(), Self::Error> {
        self.0.write_frame(block, address, data)
    }
}
