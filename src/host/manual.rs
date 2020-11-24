use crate::bus::ActiveBus;
use crate::host::{Host, HostConfig};
use crate::MacAddress;
use embedded_nal::Ipv4Addr;

pub struct Manual {
    is_setup: bool,
    settings: HostConfig,
    current: HostConfig,
}

impl Manual {
    pub fn new(mac: MacAddress, ip: Ipv4Addr, gateway: Ipv4Addr, subnet: Ipv4Addr) -> Self {
        Self {
            is_setup: false,
            settings: HostConfig {
                mac,
                ip,
                gateway,
                subnet,
            },
            current: HostConfig::default(),
        }
    }
}

impl Host for Manual {
    /// Gets (if necessary) and sets the host settings on the chip
    fn refresh<SpiBus: ActiveBus>(&mut self, bus: &mut SpiBus) -> Result<(), SpiBus::Error> {
        if !self.is_setup {
            Self::write_settings(bus, &mut self.current, &self.settings)?;
            self.is_setup = true;
        }
        Ok(())
    }
}
