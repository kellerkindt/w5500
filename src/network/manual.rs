use crate::bus::ActiveBus;
use crate::network::Network;
use crate::network::NetworkSettings;
use crate::{IpAddress, MacAddress};

pub struct Manual {
    is_setup: bool,
    settings: NetworkSettings,
    current: NetworkSettings,
}

impl Manual {
    pub fn new(mac: MacAddress, ip: IpAddress, gateway: IpAddress, subnet: IpAddress) -> Self {
        Self {
            is_setup: false,
            settings: NetworkSettings {
                mac,
                ip,
                gateway,
                subnet,
            },
            current: NetworkSettings::default(),
        }
    }
}

impl Network for Manual {
    /// Gets (if necessary) and sets the network settings on the chip
    fn refresh<SpiBus: ActiveBus>(&mut self, bus: &mut SpiBus) -> Result<(), SpiBus::Error> {
        if !self.is_setup {
            Self::write_settings(bus, &mut self.current, &self.settings)?;
            self.is_setup = true;
        }
        Ok(())
    }
}
