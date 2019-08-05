use embedded_hal::digital::v2::OutputPin;
use embedded_hal::spi::FullDuplex;

use crate::bus::Bus;

pub struct FourWire<ChipSelect: OutputPin> {
    cs: ChipSelect
}

impl<ChipSelect: OutputPin>
    FourWire<ChipSelect>
{
    pub fn new(cs: ChipSelect) -> Self {
        Self { cs }
    }
    pub fn release(self) -> ChipSelect {
        self.cs
    }
}

impl<Spi: FullDuplex<u8>, ChipSelect: OutputPin> Bus<Spi>
    for FourWire<ChipSelect>
{
    type Error = FourWireError<Spi::Error, ChipSelect::Error>;
    fn transfer_frame<'a, 'b>(
        spi: &'b mut Spi,
        address_phase: [u8; 2],
        control_phase: u8,
        data_phase: &'a mut [u8],
    ) -> Result<&'a mut [u8], nb::Error<Self::Error>> {
        // TODO implement transfer
        Ok(data_phase)
    }
}

pub enum FourWireError<SpiError, ChipSelectError> {
    SpiError(SpiError),
    ChipSelectError(ChipSelectError),
}
