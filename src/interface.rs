use embedded_hal::digital::v2::OutputPin;
use embedded_hal::spi::FullDuplex;
use embedded_nal::Ipv4Addr;

use crate::bus::{ActiveBus, ActiveFourWire, FourWire};
use crate::device::Device;
use crate::host::{Host, Manual};
use crate::uninitialized_device::{InitializeError, UninitializedDevice};
use crate::{MacAddress, Mode};

pub struct Interface<SpiBus: ActiveBus, HostImpl: Host> {
    pub device: Device<SpiBus, HostImpl>,
}

impl<SpiBus: ActiveBus, HostImpl: Host> Interface<SpiBus, HostImpl> {
    fn new(device: Device<SpiBus, HostImpl>) -> Self {
        Self { device }
    }

    pub fn release(self) -> Device<SpiBus, HostImpl> {
        self.device
    }
}

impl<Spi: FullDuplex<u8>, ChipSelect: OutputPin>
    Interface<ActiveFourWire<Spi, ChipSelect>, Manual>
{
    pub fn setup(
        spi: Spi,
        cs: ChipSelect,
        mac: MacAddress,
        ip: Ipv4Addr,
    ) -> Result<Self, InitializeError<<ActiveFourWire<Spi, ChipSelect> as ActiveBus>::Error>> {
        Ok(UninitializedDevice::new(FourWire::new(cs).activate(spi))
            .initialize_manual(mac, ip, Mode::default())?
            .into_interface())
    }
}

impl<SpiBus: ActiveBus, HostImpl: Host> From<Device<SpiBus, HostImpl>>
    for Interface<SpiBus, HostImpl>
{
    fn from(device: Device<SpiBus, HostImpl>) -> Interface<SpiBus, HostImpl> {
        Interface::<SpiBus, HostImpl>::new(device)
    }
}
