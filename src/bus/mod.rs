use embedded_hal::spi::FullDuplex;

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
        &mut self,
        block: u8,
        address: u16,
        is_write: bool,
        data: &'a mut [u8],
    ) -> Result<&'a mut [u8], Self::Error>;

    fn transfer_bytes<'a, Spi: FullDuplex<u8>>(
        spi: &mut Spi,
        bytes: &'a mut [u8],
    ) -> Result<&'a mut [u8], Spi::Error> {
        for byte in bytes.iter_mut() {
            Self::transfer_byte(spi, byte)?;
        }
        Ok(bytes)
    }

    fn transfer_byte<'a, Spi: FullDuplex<u8>>(
        spi: &mut Spi,
        byte: &'a mut u8,
    ) -> Result<&'a mut u8, Spi::Error> {
        *byte = block!(spi.send(*byte)).and_then(|_| block!(spi.read()))?;
        Ok(byte)
    }
}
