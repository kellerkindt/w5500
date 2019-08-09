use crate::bus::ActiveBus;
use crate::network::{Network, NetworkSettings};
use crate::MacAddress;

pub struct Dhcp {
    settings: NetworkSettings,
    current: NetworkSettings,
}

impl Dhcp {
    pub fn new(mac: MacAddress) -> Self {
        let settings = NetworkSettings {
            mac,
            ..NetworkSettings::default()
        };
        Self {
            settings,
            current: NetworkSettings::default(),
        }
    }
}

impl Network for Dhcp {
    /// Gets (if necessary) and sets the network settings on the chip
    fn refresh<SpiBus: ActiveBus>(&mut self, _bus: &mut SpiBus) -> Result<(), SpiBus::Error> {
        // TODO actually negotiate settings from DHCP
        Ok(())
    }
}
