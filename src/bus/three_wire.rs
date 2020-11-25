#![allow(clippy::inconsistent_digit_grouping, clippy::unusual_byte_groupings)]

use core::fmt;
use embedded_hal::spi::FullDuplex;

use crate::bus::{ActiveBus, Bus};

const WRITE_MODE_MASK: u8 = 0b00000_1_0;

const FIXED_DATA_LENGTH_MODE_1: u8 = 0b000000_01;
const FIXED_DATA_LENGTH_MODE_2: u8 = 0b000000_10;
const FIXED_DATA_LENGTH_MODE_4: u8 = 0b000000_11;

pub struct ThreeWire {}

impl ThreeWire {
    pub fn new() -> Self {
        Self {}
    }
}

impl Default for ThreeWire {
    fn default() -> Self {
        Self::new()
    }
}

impl Bus for ThreeWire {}

impl ThreeWire {
    pub fn activate<Spi: FullDuplex<u8>>(self, spi: Spi) -> ActiveThreeWire<Spi> {
        ActiveThreeWire { spi }
    }
}

pub struct ActiveThreeWire<Spi: FullDuplex<u8>> {
    spi: Spi,
}

impl<Spi: FullDuplex<u8>> ActiveBus for ActiveThreeWire<Spi> {
    type Error = ThreeWireError<Spi::Error>;

    /// Transfers a frame with an arbitrary data length in FDM
    ///
    /// This is done by passing chunks of fixed length 4, 2, or 1.  For example if a frame looks like this:
    ///
    /// (address 23) 0xF0 0xAB 0x83 0xB2 0x44 0x2C 0xAA
    ///
    /// This will be sent as separate frames in the chunks
    ///
    /// (address 23) 0xF0 0xAB 0x83 0xB2
    /// (address 27) 44 2C
    /// (address 29) AA
    fn read_frame(
        &mut self,
        block: u8,
        mut address: u16,
        data: &mut [u8],
    ) -> Result<(), Self::Error> {
        let mut control_phase = block << 3;

        let mut data_phase = &mut data[..];
        let mut last_length_written: u16;
        while !data_phase.is_empty() {
            if data_phase.len() >= 4 {
                control_phase |= FIXED_DATA_LENGTH_MODE_4;
                last_length_written = 4;
            } else if data_phase.len() >= 2 {
                control_phase |= FIXED_DATA_LENGTH_MODE_2;
                last_length_written = 2;
            } else {
                control_phase |= FIXED_DATA_LENGTH_MODE_1;
                last_length_written = 1;
            }

            let address_phase = address.to_be_bytes();
            Self::write_bytes(&mut self.spi, &address_phase)
                .and_then(|_| Self::transfer_byte(&mut self.spi, control_phase))
                .and_then(|_| {
                    Self::read_bytes(
                        &mut self.spi,
                        &mut data_phase[..last_length_written as usize],
                    )
                })?;

            address += last_length_written;
            data_phase = &mut data_phase[last_length_written as usize..];
        }
        Ok(())
    }
    fn write_frame(&mut self, block: u8, mut address: u16, data: &[u8]) -> Result<(), Self::Error> {
        let mut control_phase = block << 3 | WRITE_MODE_MASK;

        let mut data_phase = &data[..];
        let mut last_length_written: u16;
        while !data_phase.is_empty() {
            if data_phase.len() >= 4 {
                control_phase |= FIXED_DATA_LENGTH_MODE_4;
                last_length_written = 4;
            } else if data_phase.len() >= 2 {
                control_phase |= FIXED_DATA_LENGTH_MODE_2;
                last_length_written = 2;
            } else {
                control_phase |= FIXED_DATA_LENGTH_MODE_1;
                last_length_written = 1;
            }

            let address_phase = address.to_be_bytes();
            Self::write_bytes(&mut self.spi, &address_phase)
                .and_then(|_| Self::transfer_byte(&mut self.spi, control_phase))
                .and_then(|_| {
                    Self::write_bytes(&mut self.spi, &data_phase[..last_length_written as usize])
                })?;

            address += last_length_written;
            data_phase = &data_phase[last_length_written as usize..];
        }
        Ok(())
    }
}

impl<Spi: FullDuplex<u8>> ActiveThreeWire<Spi> {
    pub fn deactivate(self) -> (ThreeWire, Spi) {
        (ThreeWire::new(), self.spi)
    }
}

pub enum ThreeWireError<SpiError> {
    SpiError(SpiError),
}

impl<SpiError> From<SpiError> for ThreeWireError<SpiError> {
    fn from(error: SpiError) -> ThreeWireError<SpiError> {
        ThreeWireError::SpiError(error)
    }
}

impl<SpiError> fmt::Debug for ThreeWireError<SpiError> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "ThreeWireError::{}",
            match self {
                Self::SpiError(_) => "SpiError",
            }
        )
    }
}
