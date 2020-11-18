use core::fmt::Debug;
use embedded_hal::spi::FullDuplex;

mod four_wire;
mod three_wire;

pub use self::four_wire::ActiveFourWire;
pub use self::four_wire::FourWire;
pub use self::three_wire::ActiveThreeWire;
pub use self::three_wire::ThreeWire;

pub trait Bus {}

pub trait ActiveBus {
    type Error: Debug;

    fn read_frame(&mut self, block: u8, address: u16, data: &mut [u8]) -> Result<(), Self::Error>;

    fn write_frame(&mut self, block: u8, address: u16, data: &[u8]) -> Result<(), Self::Error>;

    fn read_bytes<Spi: FullDuplex<u8>>(spi: &mut Spi, bytes: &mut [u8]) -> Result<(), Spi::Error> {
        for byte in bytes.iter_mut() {
            *byte = Self::transfer_byte(spi, *byte)?;
        }
        Ok(())
    }

    fn write_bytes<Spi: FullDuplex<u8>>(spi: &mut Spi, bytes: &[u8]) -> Result<(), Spi::Error> {
        for byte in bytes.iter() {
            Self::transfer_byte(spi, *byte)?;
        }
        Ok(())
    }

    fn transfer_byte<Spi: FullDuplex<u8>>(spi: &mut Spi, byte: u8) -> Result<u8, Spi::Error> {
        block!(spi.send(byte)).and_then(|_| block!(spi.read()))
    }
}
