mod dhcp;
mod manual;

pub use self::dhcp::Dhcp;
pub use self::manual::Manual;
use crate::bus::Bus;
use crate::register;
use crate::MacAddress;
use embedded_nal::Ipv4Addr;

pub struct HostConfig {
    mac: MacAddress,
    ip: Ipv4Addr,
    gateway: Ipv4Addr,
    subnet: Ipv4Addr,
}

impl Default for HostConfig {
    fn default() -> Self {
        Self {
            mac: MacAddress::default(),
            ip: Ipv4Addr::unspecified(),
            gateway: Ipv4Addr::unspecified(),
            subnet: Ipv4Addr::unspecified(),
        }
    }
}

pub trait Host {
    /// Gets (if necessary) and sets the host settings on the chip
    fn refresh<SpiBus: Bus>(&mut self, bus: &mut SpiBus) -> Result<(), SpiBus::Error>;

    /// Write changed settings to chip
    ///
    /// Will check all settings and write any new ones to the chip.  Will update the settings returned by `current`
    /// with any changes.
    fn write_settings<SpiBus: Bus>(
        bus: &mut SpiBus,
        current: &mut HostConfig,
        settings: &HostConfig,
    ) -> Result<(), SpiBus::Error> {
        if settings.gateway != current.gateway {
            let address = settings.gateway.octets();
            bus.write_frame(register::COMMON, register::common::GATEWAY, &address)?;
            current.gateway = settings.gateway;
        }
        if settings.subnet != current.subnet {
            let address = settings.subnet.octets();
            bus.write_frame(register::COMMON, register::common::SUBNET_MASK, &address)?;
            current.subnet = settings.subnet;
        }
        if settings.mac != current.mac {
            let address = settings.mac.octets;
            bus.write_frame(register::COMMON, register::common::MAC, &address)?;
            current.mac = settings.mac;
        }
        if settings.ip != current.ip {
            let address = settings.ip.octets();
            bus.write_frame(register::COMMON, register::common::IP, &address)?;
            current.ip = settings.ip;
        }
        Ok(())
    }
}
