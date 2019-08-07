use bus::{ActiveBus, ActiveFourWire, ActiveThreeWire};
use embedded_hal::digital::v2::OutputPin;
use embedded_hal::spi::FullDuplex;
use w5500::W5500;

pub struct UninitializedW5500<SpiBus: ActiveBus> {
    bus: SpiBus,
}

impl<SpiBus: ActiveBus> UninitializedW5500<SpiBus> {
    pub fn initialize(self) -> W5500<SpiBus> {
        // TODO actually initialize chip
        W5500::new(self.bus)
    }
    pub fn new(bus: SpiBus) -> UninitializedW5500<SpiBus> {
        UninitializedW5500 { bus: bus }
    }
}

impl<Spi: FullDuplex<u8>, ChipSelect: OutputPin>
    UninitializedW5500<ActiveFourWire<Spi, ChipSelect>>
{
    pub fn deactivate(self) -> (Spi, ChipSelect) {
        let (bus, spi) = self.bus.deactivate();
        (spi, bus.release())
    }
}

impl<Spi: FullDuplex<u8>> UninitializedW5500<ActiveThreeWire<Spi>> {
    pub fn deactivate(self) -> Spi {
        self.bus.deactivate().1
    }
}
