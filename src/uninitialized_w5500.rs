use bus::{ActiveBus, ActiveFourWire, ActiveThreeWire};
use embedded_hal::digital::v2::OutputPin;
use embedded_hal::spi::FullDuplex;
use w5500::W5500;
use crate::Settings;
use register;

pub struct UninitializedW5500<SpiBus: ActiveBus> {
    bus: SpiBus,
}

impl<SpiBus: ActiveBus> UninitializedW5500<SpiBus> {
    pub fn initialize(mut self, settings: Settings) -> Result<W5500<SpiBus>, SpiBus::Error> {
        self.set_mode(settings)?;
        // TODO set up IP/etc
        // TODO give ownership of all sockets
        Ok(W5500::new(self.bus))
    }
    pub fn new(bus: SpiBus) -> UninitializedW5500<SpiBus> {
        UninitializedW5500 { bus: bus }
    }

    fn set_mode(
        &mut self,
        settings: Settings,
    ) -> Result<(), SpiBus::Error> {
        let mut mode = [0];
        mode[0] |= settings.on_wake_on_lan as u8;
        mode[0] |= settings.on_ping_request as u8;
        mode[0] |= settings.connection_type as u8;
        mode[0] |= settings.arp_responses as u8;
        block!(self.bus.transfer_frame(register::COMMON, register::common::MODE, true, &mut mode))?;
        Ok(())
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
