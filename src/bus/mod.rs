use nb::Result;

mod four_wire;
mod three_wire;

pub use self::four_wire::ActiveFourWire;
pub use self::four_wire::FourWire;
pub use self::three_wire::ActiveThreeWire;
pub use self::three_wire::ThreeWire;

pub trait Bus {}

pub trait ActiveBus {
    type Error;
    fn transfer_frame<'a>(
        address_phase: [u8; 2],
        control_phase: u8,
        data_phase: &'a mut [u8],
    ) -> Result<&'a mut [u8], Self::Error>;
}
