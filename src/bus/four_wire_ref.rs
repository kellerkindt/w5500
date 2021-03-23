#![allow(clippy::inconsistent_digit_grouping, clippy::unusual_byte_groupings)]

use core::fmt;
use embedded_hal::blocking::spi::{Transfer, Write};
use embedded_hal::digital::v2::OutputPin;

use crate::bus::{Bus, FourWire, FourWireError};

const WRITE_MODE_MASK: u8 = 0b00000_1_00;

// TODO This name is not ideal, should be renamed to VDM
/// This is just like [crate::bus::FourWire] but takes references instead of ownership
/// for the SPI bus and the ChipSelect pin
pub struct FourWireRef<'a, Spi: Transfer<u8> + Write<u8>, ChipSelect: OutputPin>(
    FourWire<SpiRef<'a, Spi>, OutputPinRef<'a, ChipSelect>>,
);

impl<'a, Spi: Transfer<u8> + Write<u8>, ChipSelect: OutputPin> FourWireRef<'a, Spi, ChipSelect> {
    pub fn new(spi: &'a mut Spi, cs: &'a mut ChipSelect) -> Self {
        Self(FourWire::new(SpiRef(spi), OutputPinRef(cs)))
    }

    // this is actually a bit silly, but maybe someday someone finds this useful
    pub fn release(self) -> (&'a mut Spi, &'a mut ChipSelect) {
        let (spi_ref, cs_ref) = self.0.release();
        (spi_ref.0, cs_ref.0)
    }
}

impl<Spi: Transfer<u8> + Write<u8>, ChipSelect: OutputPin> Bus
    for FourWireRef<'_, Spi, ChipSelect>
{
    type Error =
        FourWireError<<Spi as Transfer<u8>>::Error, <Spi as Write<u8>>::Error, ChipSelect::Error>;

    #[inline]
    fn read_frame(&mut self, block: u8, address: u16, data: &mut [u8]) -> Result<(), Self::Error> {
        self.0.read_frame(block, address, data)
    }

    #[inline]
    fn write_frame(&mut self, block: u8, address: u16, data: &[u8]) -> Result<(), Self::Error> {
        self.0.write_frame(block, address, data)
    }
}

#[derive(Debug)]
pub struct SpiRef<'a, Spi: Transfer<u8> + Write<u8>>(&'a mut Spi);

impl<'a, Spi: Transfer<u8> + Write<u8>> Transfer<u8> for SpiRef<'a, Spi> {
    type Error = <Spi as Transfer<u8>>::Error;

    #[inline]
    fn transfer<'w>(&mut self, words: &'w mut [u8]) -> Result<&'w [u8], Self::Error> {
        self.0.transfer(words)
    }
}

impl<'a, Spi: Transfer<u8> + Write<u8>> Write<u8> for SpiRef<'a, Spi> {
    type Error = <Spi as Write<u8>>::Error;

    #[inline]
    fn write(&mut self, words: &[u8]) -> Result<(), Self::Error> {
        self.0.write(words)
    }
}

#[derive(Debug)]
pub struct OutputPinRef<'a, P: OutputPin>(&'a mut P);

impl<'a, P: OutputPin> OutputPin for OutputPinRef<'a, P> {
    type Error = P::Error;

    #[inline]
    fn set_low(&mut self) -> Result<(), Self::Error> {
        self.0.set_low()
    }

    #[inline]
    fn set_high(&mut self) -> Result<(), Self::Error> {
        self.0.set_high()
    }
}
