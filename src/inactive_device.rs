use crate::bus::{ActiveFourWire, ActiveThreeWire, Bus, FourWire, ThreeWire};
use crate::device::Device;
use crate::host::Host;
use embedded_hal::digital::v2::OutputPin;
use embedded_hal::spi::FullDuplex;
use network::Network;

pub struct InactiveDevice<SpiBus: Bus, HostImpl: Host> {
    bus: SpiBus,
    host: HostImpl,
}

impl<SpiBus: Bus, HostImpl: Host> InactiveDevice<SpiBus, HostImpl> {
    pub fn new(bus: SpiBus, host: HostImpl) -> Self {
        Self { bus, host }
    }
}

impl<ChipSelect: OutputPin, HostImpl: Host> InactiveDevice<FourWire<ChipSelect>, HostImpl> {
    pub fn activate<Spi: FullDuplex<u8>>(
        self,
        spi: Spi,
    ) -> Device<ActiveFourWire<Spi, ChipSelect>, HostImpl> {
        Device::new(self.bus.activate(spi), self.host)
    }
}

impl<HostImpl: Host> InactiveDevice<ThreeWire, HostImpl> {
    pub fn activate<Spi: FullDuplex<u8>>(self, spi: Spi) -> Device<ActiveThreeWire<Spi>, HostImpl> {
        Device::new(self.bus.activate(spi), self.host)
    }
}
