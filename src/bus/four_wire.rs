use byteorder::{BigEndian, ByteOrder};
use embedded_hal::digital::v2::OutputPin;
use embedded_hal::spi::FullDuplex;

use crate::bus::{ActiveBus, Bus};

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
    fn transfer_frame<'a>(
        &mut self,
        address_phase: u16,
        mut control_phase: u8,
        data_phase: &'a mut [u8],
    ) -> Result<&'a mut [u8], nb::Error<Self::Error>> {
        let mut address_bytes = [0u8; 2];
        BigEndian::write_u16(&mut address_bytes, address_phase);
        self.cs
            .set_high()
            .map_err(|e| Self::Error::ChipSelectError(e))?;
        block!(Self::transfer_bytes(&mut self.spi, &mut address_bytes)
            .and_then(|_| Self::transfer_byte(&mut self.spi, &mut control_phase))
            .and_then(|_| Self::transfer_bytes(&mut self.spi, data_phase)))
        .map_err(|e| Self::Error::SpiError(e))?;
        self.cs
            .set_low()
            .map_err(|e| Self::Error::ChipSelectError(e))?;
        Ok(data_phase)
    }
}
impl<Spi: FullDuplex<u8>, ChipSelect: OutputPin> ActiveFourWire<Spi, ChipSelect> {
    pub fn deactivate(self) -> (FourWire<ChipSelect>, Spi) {
        (FourWire::new(self.cs), self.spi)
    }
}

pub enum FourWireError<SpiError, ChipSelectError> {
    SpiError(SpiError),
    ChipSelectError(ChipSelectError),
}
