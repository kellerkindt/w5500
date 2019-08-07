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
        address_phase: [u8; 2],
        control_phase: u8,
        data_phase: &'a mut [u8],
    ) -> Result<&'a mut [u8], nb::Error<Self::Error>> {
        // TODO implement transfer
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
