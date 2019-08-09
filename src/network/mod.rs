mod dhcp;
mod manual;

pub use self::dhcp::Dhcp;
pub use self::manual::Manual;
use crate::bus::ActiveBus;
use crate::register;
use crate::{IpAddress, MacAddress};

#[derive(Default)]
pub struct NetworkSettings {
    mac: MacAddress,
    ip: IpAddress,
    gateway: IpAddress,
    subnet: IpAddress,
}

pub trait Network {
    /// Gets (if necessary) and sets the network settings on the chip
    fn refresh<SpiBus: ActiveBus>(&mut self, bus: &mut SpiBus) -> Result<(), SpiBus::Error>;

    /// Write changed settings to chip
    ///
    /// Will check all settings and write any new ones to the chip.  Will update the settings returned by `current`
    /// with any changes.
    fn write_settings<SpiBus: ActiveBus>(
        bus: &mut SpiBus,
        current: &mut NetworkSettings,
        settings: &NetworkSettings,
    ) -> Result<(), SpiBus::Error> {
        if settings.gateway != current.gateway {
            let mut address = settings.gateway.address;
            block!(bus.transfer_frame(
                register::COMMON,
                register::common::GATEWAY,
                true,
                &mut address
            ))?;
            current.gateway = settings.gateway;
        }
        if settings.subnet != current.subnet {
            let mut address = settings.subnet.address;
            block!(bus.transfer_frame(
                register::COMMON,
                register::common::SUBNET_MASK,
                true,
                &mut address
            ))?;
            current.subnet = settings.subnet;
        }
        if settings.mac != current.mac {
            let mut address = settings.mac.address;
            block!(bus.transfer_frame(
                register::COMMON,
                register::common::MAC,
                true,
                &mut address
            ))?;
            current.mac = settings.mac;
        }
        if settings.ip != current.ip {
            let mut address = settings.ip.address;
            block!(bus.transfer_frame(register::COMMON, register::common::IP, true, &mut address))?;
            current.ip = settings.ip;
        }
        Ok(())
    }
}
