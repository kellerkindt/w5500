use crate::inactive_w5500::InactiveW5500;
use crate::uninitialized_w5500::UninitializedW5500;
use bus::{ActiveBus, ActiveFourWire, ActiveThreeWire, FourWire, ThreeWire};
use network::Network;
use embedded_hal::digital::v2::OutputPin;
use embedded_hal::spi::FullDuplex;
use register;

pub struct W5500<SpiBus: ActiveBus, NetworkImpl: Network> {
    bus: SpiBus,
    network: NetworkImpl,
}

impl<SpiBus: ActiveBus, NetworkImpl: Network> W5500<SpiBus, NetworkImpl> {
    pub fn new(bus: SpiBus, network: NetworkImpl) -> Self {
        W5500 { bus, network }
    }
    pub fn reset(mut self) -> Result<UninitializedW5500<SpiBus>, SpiBus::Error> {
        // TODO accept all sockets back
        self.clear_mode()?;
        Ok(UninitializedW5500::new(self.bus))
    }

    fn clear_mode(&mut self) -> Result<(), SpiBus::Error> {
        // reset bit
        let mut mode = [0b10000000];
        block!(self.bus.transfer_frame(register::COMMON, register::common::MODE, true, &mut mode))?;
        Ok(())
    }

    //TODO open_udp_socket
}

impl<Spi: FullDuplex<u8>, ChipSelect: OutputPin, NetworkImpl: Network> W5500<ActiveFourWire<Spi, ChipSelect>, NetworkImpl> {
    pub fn deactivate(self) -> (InactiveW5500<FourWire<ChipSelect>, NetworkImpl>, Spi) {
        let (bus, spi) = self.bus.deactivate();
        (InactiveW5500::new(bus, self.network), spi)
    }
}

impl<Spi: FullDuplex<u8>, NetworkImpl: Network> W5500<ActiveThreeWire<Spi>, NetworkImpl> {
    pub fn deactivate(self) -> (InactiveW5500<ThreeWire, NetworkImpl>, Spi) {
        let (bus, spi) = self.bus.deactivate();
        (InactiveW5500::new(bus, self.network), spi)
    }
}
