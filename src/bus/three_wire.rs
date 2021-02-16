#![allow(clippy::inconsistent_digit_grouping, clippy::unusual_byte_groupings)]

use core::fmt;
use embedded_hal::blocking::spi::{Transfer, Write};

use crate::bus::Bus;

const WRITE_MODE_MASK: u8 = 0b00000_1_0;

const FIXED_DATA_LENGTH_MODE_1: u8 = 0b000000_01;
const FIXED_DATA_LENGTH_MODE_2: u8 = 0b000000_10;
const FIXED_DATA_LENGTH_MODE_4: u8 = 0b000000_11;

pub struct ThreeWire<Spi: Transfer<u8> + Write<u8>> {
    spi: Spi,
}

impl<Spi: Transfer<u8> + Write<u8>> ThreeWire<Spi> {
    pub fn new(spi: Spi) -> Self {
        Self { spi }
    }

    pub fn release(self) -> Spi {
        self.spi
    }
}

impl<Spi: Transfer<u8> + Write<u8>> Bus for ThreeWire<Spi> {
    type Error = ThreeWireError<<Spi as Transfer<u8>>::Error, <Spi as Write<u8>>::Error>;

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
            self.spi
                .write(&address_phase)
                .and_then(|_| self.spi.write(&[control_phase]))
                .map_err(ThreeWireError::WriteError)?;
            self.spi
                .transfer(&mut data_phase[..last_length_written as usize])
                .map_err(ThreeWireError::TransferError)?;

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
            self.spi
                .write(&address_phase)
                .and_then(|_| self.spi.write(&[control_phase]))
                .and_then(|_| self.spi.write(&data_phase[..last_length_written as usize]))
                .map_err(ThreeWireError::WriteError)?;

            address += last_length_written;
            data_phase = &data_phase[last_length_written as usize..];
        }
        Ok(())
    }
}

// Must use map_err, ambiguity prevents From from being implemented
pub enum ThreeWireError<TransferError, WriteError> {
    TransferError(TransferError),
    WriteError(WriteError),
}

impl<TransferError, WriteError> fmt::Debug for ThreeWireError<TransferError, WriteError> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "ThreeWireError::{}",
            match self {
                Self::TransferError(_) => "TransferError",
                Self::WriteError(_) => "WriteError",
            }
        )
    }
}
