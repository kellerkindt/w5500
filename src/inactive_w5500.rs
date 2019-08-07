use bus::{ActiveFourWire, ActiveThreeWire, Bus, FourWire, ThreeWire};
use embedded_hal::digital::v2::OutputPin;
use embedded_hal::spi::FullDuplex;
use w5500::W5500;

pub struct InactiveW5500<SpiBus: Bus> {
    bus: SpiBus,
}

impl<SpiBus: Bus> InactiveW5500<SpiBus> {
    pub fn new(bus: SpiBus) -> Self {
        Self { bus }
    }
}

impl<ChipSelect: OutputPin> InactiveW5500<FourWire<ChipSelect>> {
    pub fn activate<Spi: FullDuplex<u8>>(self, spi: Spi) -> W5500<ActiveFourWire<Spi, ChipSelect>> {
        W5500::new(self.bus.activate(spi))
    }
}

impl InactiveW5500<ThreeWire> {
    pub fn activate<Spi: FullDuplex<u8>>(self, spi: Spi) -> W5500<ActiveThreeWire<Spi>> {
        W5500::new(self.bus.activate(spi))
    }
}
