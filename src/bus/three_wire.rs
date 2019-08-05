use embedded_hal::spi::FullDuplex;

use crate::bus::Bus;

pub struct ThreeWire {}

impl ThreeWire {
    pub fn new() -> Self {
        Self { }
    }
}

impl<Spi: FullDuplex<u8>> Bus<Spi> for ThreeWire {
    type Error = Spi::Error;
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
