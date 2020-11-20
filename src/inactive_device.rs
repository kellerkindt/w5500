use bus::{ActiveFourWire, ActiveThreeWire, Bus, FourWire, ThreeWire};
use embedded_hal::digital::v2::OutputPin;
use embedded_hal::spi::FullDuplex;
use network::Network;
use device::Device;

pub struct InactiveDevice<SpiBus: Bus, NetworkImpl: Network> {
    bus: SpiBus,
    network: NetworkImpl,
}

impl<SpiBus: Bus, NetworkImpl: Network> InactiveDevice<SpiBus, NetworkImpl> {
    pub fn new(bus: SpiBus, network: NetworkImpl) -> Self {
        Self { bus, network }
    }
}

impl<ChipSelect: OutputPin, NetworkImpl: Network> InactiveDevice<FourWire<ChipSelect>, NetworkImpl> {
    pub fn activate<Spi: FullDuplex<u8>>(
        self,
        spi: Spi,
    ) -> Device<ActiveFourWire<Spi, ChipSelect>, NetworkImpl> {
        Device::new(self.bus.activate(spi), self.network)
    }
}

impl<NetworkImpl: Network> InactiveDevice<ThreeWire, NetworkImpl> {
    pub fn activate<Spi: FullDuplex<u8>>(
        self,
        spi: Spi,
    ) -> Device<ActiveThreeWire<Spi>, NetworkImpl> {
        Device::new(self.bus.activate(spi), self.network)
    }
}
