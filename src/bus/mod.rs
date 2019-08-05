use nb::Result;
use embedded_hal::spi::FullDuplex;

mod four_wire;
mod three_wire;

pub use self::four_wire::FourWire;
pub use self::three_wire::ThreeWire;

pub trait Bus<Spi: FullDuplex<u8>> {
    type Error;
    fn transfer_frame<'a, 'b>(
        spi: &'b mut Spi,
        address_phase: [u8; 2],
        control_phase: u8,
        data_phase: &'a mut [u8],
    ) -> Result<&'a mut [u8], Self::Error>;
}
