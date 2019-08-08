use embedded_hal::spi::FullDuplex;

use crate::bus::{ActiveBus, Bus};

pub struct ThreeWire {}

impl ThreeWire {
    pub fn new() -> Self {
        Self {}
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
    type Error = Spi::Error;
    fn transfer_frame<'a>(
        &mut self,
        address_phase: u16,
        control_phase: u8,
        data_phase: &'a mut [u8],
    ) -> Result<&'a mut [u8], nb::Error<Self::Error>> {
        // TODO implement transfer
        Ok(data_phase)
    }
}

impl<Spi: FullDuplex<u8>> ActiveThreeWire<Spi> {
    pub fn deactivate(self) -> (ThreeWire, Spi) {
        (ThreeWire::new(), self.spi)
    }
}
