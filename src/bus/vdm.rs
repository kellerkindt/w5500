#![allow(clippy::inconsistent_digit_grouping, clippy::unusual_byte_groupings)]

use core::fmt;
use embedded_hal::blocking::spi::{Transfer, Write};
use embedded_hal::digital::v2::OutputPin;

use crate::bus::Bus;

const WRITE_MODE_MASK: u8 = 0b00000_1_00;

/// Variable Data Mode for communicating with the chip.  Requires four connection wires:
///
/// - MISO
/// - MOSI
/// - CLK
/// - CS
pub struct Vdm<Spi: Transfer<u8> + Write<u8>, ChipSelect: OutputPin> {
    cs: ChipSelect,
    spi: Spi,
}

impl<Spi: Transfer<u8> + Write<u8>, ChipSelect: OutputPin> Vdm<Spi, ChipSelect> {
    pub fn new(spi: Spi, cs: ChipSelect) -> Self {
        Self { cs, spi }
    }

    pub fn release(self) -> (Spi, ChipSelect) {
        (self.spi, self.cs)
    }
}

impl<Spi: Transfer<u8> + Write<u8>, ChipSelect: OutputPin> Bus for Vdm<Spi, ChipSelect> {
    type Error =
        VdmError<<Spi as Transfer<u8>>::Error, <Spi as Write<u8>>::Error, ChipSelect::Error>;
    fn read_frame(&mut self, block: u8, address: u16, data: &mut [u8]) -> Result<(), Self::Error> {
        let address_phase = address.to_be_bytes();
        let control_phase = block << 3;
        let data_phase = data;
        self.cs.set_low().map_err(VdmError::ChipSelectError)?;
        self.spi
            .write(&address_phase)
            .and_then(|_| self.spi.write(&[control_phase]))
            .map_err(VdmError::WriteError)?;
        self.spi
            .transfer(data_phase)
            .map_err(VdmError::TransferError)?;
        self.cs.set_high().map_err(VdmError::ChipSelectError)?;

        Ok(())
    }
    fn write_frame(&mut self, block: u8, address: u16, data: &[u8]) -> Result<(), Self::Error> {
        let address_phase = address.to_be_bytes();
        let control_phase = block << 3 | WRITE_MODE_MASK;
        let data_phase = data;
        self.cs.set_low().map_err(VdmError::ChipSelectError)?;
        self.spi
            .write(&address_phase)
            .and_then(|_| self.spi.write(&[control_phase]))
            .and_then(|_| self.spi.write(data_phase))
            .map_err(VdmError::WriteError)?;
        self.cs.set_high().map_err(VdmError::ChipSelectError)?;

        Ok(())
    }
}

// Must use map_err, ambiguity prevents From from being implemented
#[repr(u8)]
pub enum VdmError<TransferError, WriteError, ChipSelectError> {
    TransferError(TransferError),
    WriteError(WriteError),
    ChipSelectError(ChipSelectError),
}

impl<TransferError, WriteError, ChipSelectError> fmt::Debug
    for VdmError<TransferError, WriteError, ChipSelectError>
{
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "VdmError::{}",
            match self {
                Self::TransferError(_) => "TransferError",
                Self::WriteError(_) => "WriteError",
                Self::ChipSelectError(_) => "ChipSelectError",
            }
        )
    }
}

// TODO Improved error rendering could be done with specialization.
// https://github.com/rust-lang/rust/issues/31844
