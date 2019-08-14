use bus::{ActiveFourWire, ActiveThreeWire, Bus, FourWire, ThreeWire};
use embedded_hal::digital::v2::OutputPin;
use embedded_hal::spi::FullDuplex;
use network::Network;
use socket::OwnedSockets;
use w5500::W5500;

pub struct InactiveW5500<SpiBus: Bus, NetworkImpl: Network> {
    bus: SpiBus,
    network: NetworkImpl,
    sockets: OwnedSockets,
}

impl<SpiBus: Bus, NetworkImpl: Network> InactiveW5500<SpiBus, NetworkImpl> {
    pub fn new(bus: SpiBus, network: NetworkImpl, sockets: OwnedSockets) -> Self {
        Self {
            bus,
            network,
            sockets,
        }
    }
}

impl<ChipSelect: OutputPin, NetworkImpl: Network> InactiveW5500<FourWire<ChipSelect>, NetworkImpl> {
    pub fn activate<Spi: FullDuplex<u8>>(
        self,
        spi: Spi,
    ) -> W5500<ActiveFourWire<Spi, ChipSelect>, NetworkImpl> {
        W5500::new(self.bus.activate(spi), self.network, self.sockets)
    }
}

impl<NetworkImpl: Network> InactiveW5500<ThreeWire, NetworkImpl> {
    pub fn activate<Spi: FullDuplex<u8>>(
        self,
        spi: Spi,
    ) -> W5500<ActiveThreeWire<Spi>, NetworkImpl> {
        W5500::new(self.bus.activate(spi), self.network, self.sockets)
    }
}
