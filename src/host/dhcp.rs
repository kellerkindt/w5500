use crate::bus::ActiveBus;
use crate::host::Host;
use crate::MacAddress;

pub struct Dhcp {
    // settings: HostConfig,
// current: HostConfig,
}

impl Dhcp {
    pub fn new(_mac: MacAddress) -> Self {
        // let settings = HostConfig {
        //     mac,
        //     ..HostConfig::default()
        // };
        Self {
            // settings,
            // current: HostConfig::default(),
        }
    }
}

impl Host for Dhcp {
    /// Gets (if necessary) and sets the host settings on the chip
    fn refresh<SpiBus: ActiveBus>(&mut self, _bus: &mut SpiBus) -> Result<(), SpiBus::Error> {
        // TODO actually negotiate settings from DHCP
        // TODO figure out how should receive socket for DHCP negotiations
        Ok(())
    }
}
