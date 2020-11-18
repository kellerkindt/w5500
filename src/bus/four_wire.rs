use core::fmt;
use embedded_hal::digital::v2::OutputPin;
use embedded_hal::spi::FullDuplex;

use crate::bus::{ActiveBus, Bus};

const WRITE_MODE_MASK: u8 = 0b00000_1_00;

pub struct FourWire<ChipSelect: OutputPin> {
    cs: ChipSelect,
}

impl<ChipSelect: OutputPin> FourWire<ChipSelect> {
    pub fn new(cs: ChipSelect) -> Self {
        Self { cs }
    }
    pub fn release(self) -> ChipSelect {
        self.cs
    }
}

impl<ChipSelect: OutputPin> Bus for FourWire<ChipSelect> {}

impl<ChipSelect: OutputPin> FourWire<ChipSelect> {
    pub fn activate<Spi: FullDuplex<u8>>(self, spi: Spi) -> ActiveFourWire<Spi, ChipSelect> {
        ActiveFourWire { cs: self.cs, spi }
    }
}

pub struct ActiveFourWire<Spi: FullDuplex<u8>, ChipSelect: OutputPin> {
    cs: ChipSelect,
    spi: Spi,
}

impl<Spi: FullDuplex<u8>, ChipSelect: OutputPin> ActiveBus for ActiveFourWire<Spi, ChipSelect> {
    type Error = FourWireError<Spi::Error, ChipSelect::Error>;
    fn read_frame(&mut self, block: u8, address: u16, data: &mut [u8]) -> Result<(), Self::Error> {
        let address_phase = address.to_be_bytes();
        let control_phase = block << 3;
        let data_phase = data;
        self.cs
            .set_low()
            .map_err(|e| FourWireError::ChipSelectError(e))?;
        Self::write_bytes(&mut self.spi, &address_phase)
            .and_then(|_| Self::transfer_byte(&mut self.spi, control_phase))
            .and_then(|_| Self::read_bytes(&mut self.spi, data_phase))
            .map_err(|e| FourWireError::SpiError(e))?;
        self.cs
            .set_high()
            .map_err(|e| FourWireError::ChipSelectError(e))?;

        Ok(())
    }
    fn write_frame(&mut self, block: u8, address: u16, data: &[u8]) -> Result<(), Self::Error> {
        let address_phase = address.to_be_bytes();
        let control_phase = block << 3 | WRITE_MODE_MASK;
        let data_phase = data;
        self.cs
            .set_low()
            .map_err(|e| FourWireError::ChipSelectError(e))?;
        Self::write_bytes(&mut self.spi, &address_phase)
            .and_then(|_| Self::transfer_byte(&mut self.spi, control_phase))
            .and_then(|_| Self::write_bytes(&mut self.spi, data_phase))
            .map_err(|e| FourWireError::SpiError(e))?;
        self.cs
            .set_high()
            .map_err(|e| FourWireError::ChipSelectError(e))?;

        Ok(())
    }
}
impl<Spi: FullDuplex<u8>, ChipSelect: OutputPin> ActiveFourWire<Spi, ChipSelect> {
    pub fn deactivate(self) -> (FourWire<ChipSelect>, Spi) {
        (FourWire::new(self.cs), self.spi)
    }
}

#[repr(u8)]
pub enum FourWireError<SpiError, ChipSelectError> {
    SpiError(SpiError),
    ChipSelectError(ChipSelectError),
}

impl<SpiError, ChipSelectError> fmt::Debug for FourWireError<SpiError, ChipSelectError> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "FourWireError::{}",
            match self {
                Self::SpiError(_) => "SpiError",
                Self::ChipSelectError(_) => "ChipSelectError",
            }
        )
    }
}
// TODO impl From and remove map_errs
