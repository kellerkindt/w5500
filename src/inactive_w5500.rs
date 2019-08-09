use bus::{ActiveFourWire, ActiveThreeWire, Bus, FourWire, ThreeWire};
use network::Network;
use embedded_hal::digital::v2::OutputPin;
use embedded_hal::spi::FullDuplex;
use w5500::W5500;

pub struct InactiveW5500<SpiBus: Bus, NetworkImpl: Network> {
    bus: SpiBus,
    network: NetworkImpl
}

impl<SpiBus: Bus, NetworkImpl: Network> InactiveW5500<SpiBus, NetworkImpl> {
    pub fn new(bus: SpiBus, network: NetworkImpl) -> Self {
        Self { bus, network }
    }
}

impl<ChipSelect: OutputPin, NetworkImpl: Network> InactiveW5500<FourWire<ChipSelect>, NetworkImpl> {
    pub fn activate<Spi: FullDuplex<u8>>(self, spi: Spi) -> W5500<ActiveFourWire<Spi, ChipSelect>, NetworkImpl> {
        W5500::new(self.bus.activate(spi), self.network)
    }
}

impl<NetworkImpl: Network> InactiveW5500<ThreeWire, NetworkImpl> {
    pub fn activate<Spi: FullDuplex<u8>>(self, spi: Spi) -> W5500<ActiveThreeWire<Spi>, NetworkImpl> {
        W5500::new(self.bus.activate(spi), self.network)
    }
}
