use crate::inactive_w5500::InactiveW5500;
use crate::uninitialized_w5500::UninitializedW5500;
use bus::{ActiveBus, ActiveFourWire, ActiveThreeWire, FourWire, ThreeWire};
use embedded_hal::digital::v2::OutputPin;
use embedded_hal::spi::FullDuplex;

pub struct W5500<SpiBus: ActiveBus> {
    bus: SpiBus,
}

impl<SpiBus: ActiveBus> W5500<SpiBus> {
    pub fn new(bus: SpiBus) -> Self {
        W5500 { bus }
    }
    pub fn reset(self) -> UninitializedW5500<SpiBus> {
        // TODO reset chip
        UninitializedW5500::new(self.bus)
    }
    //TODO open_udp_socket
}

impl<Spi: FullDuplex<u8>, ChipSelect: OutputPin> W5500<ActiveFourWire<Spi, ChipSelect>> {
    pub fn deactivate(self) -> (InactiveW5500<FourWire<ChipSelect>>, Spi) {
        let (bus, spi) = self.bus.deactivate();
        (InactiveW5500::new(bus), spi)
    }
}

impl<Spi: FullDuplex<u8>> W5500<ActiveThreeWire<Spi>> {
    pub fn deactivate(self) -> (InactiveW5500<ThreeWire>, Spi) {
        let (bus, spi) = self.bus.deactivate();
        (InactiveW5500::new(bus), spi)
    }
}
