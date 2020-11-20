use bus::{ActiveBus};
use device::Device;
use network::Network;
use core::cell::RefCell;

pub struct Interface<SpiBus: ActiveBus, NetworkImpl: Network> {
    pub device: RefCell<Device<SpiBus, NetworkImpl>>,
}

impl<SpiBus: ActiveBus, NetworkImpl: Network> Interface<SpiBus, NetworkImpl> {
    fn new(device: Device<SpiBus, NetworkImpl>) -> Self {
        Self { device: RefCell::new(device) }
    }
}

impl<SpiBus: ActiveBus, NetworkImpl: Network> From<Device<SpiBus, NetworkImpl>> for Interface<SpiBus, NetworkImpl> {
    fn from(device: Device<SpiBus, NetworkImpl>) -> Interface<SpiBus, NetworkImpl> {
        Interface::<SpiBus, NetworkImpl>::new(device)
    }
}

