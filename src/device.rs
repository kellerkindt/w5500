use crate::inactive_device::InactiveDevice;
use crate::uninitialized_device::UninitializedDevice;
use bit_field::BitArray;
use bus::{ActiveBus, ActiveFourWire, ActiveThreeWire, FourWire, ThreeWire};
use embedded_hal::digital::v2::OutputPin;
use embedded_hal::spi::FullDuplex;

use interface::Interface;
use network::Network;
use crate::bus::{ActiveBus, ActiveFourWire, ActiveThreeWire, FourWire, ThreeWire};
use crate::host::Host;
use crate::inactive_device::InactiveDevice;
use crate::register;
use crate::socket::Socket;
use crate::uninitialized_device::UninitializedDevice;

pub struct Device<SpiBus: ActiveBus, HostImpl: Host> {
    pub bus: SpiBus,
    host: HostImpl,
    sockets: [u8; 1],
}

pub enum ResetError<E> {
    SocketsNotReleased,
    Other(E),
}

impl<E> From<E> for ResetError<E> {
    fn from(error: E) -> ResetError<E> {
        ResetError::Other(error)
    }
}

impl<SpiBus: ActiveBus, HostImpl: Host> Device<SpiBus, HostImpl> {
    pub fn new(bus: SpiBus, host: HostImpl) -> Self {
        Device {
            bus,
            host,
            sockets: [0b11111111],
        }
    }

    pub fn reset(mut self) -> Result<UninitializedDevice<SpiBus>, ResetError<SpiBus::Error>> {
        if self.sockets != [0b11111111] {
            Err(ResetError::SocketsNotReleased)
        } else {
            self.clear_mode()?;
            Ok(UninitializedDevice::new(self.bus))
        }
    }

    fn clear_mode(&mut self) -> Result<(), SpiBus::Error> {
        // reset bit
        let mode = [0b10000000];
        self.bus
            .write_frame(register::COMMON, register::common::MODE, &mode)?;
        Ok(())
    }

    pub fn take_socket(&mut self) -> Option<Socket> {
        for index in 0..8 {
            if self.sockets.get_bit(index) {
                self.sockets.set_bit(index, false);
                return Some(Socket::new(index as u8));
            }
        }
        None
    }

    pub fn phy_config(&mut self) -> Result<register::common::PhyConfig, SpiBus::Error> {
        let mut phy = [0u8];
        self.bus
            .read_frame(register::COMMON, register::common::PHY_CONFIG, &mut phy)?;
        Ok(phy[0].into())
    }

    pub fn into_interface(self) -> Interface<SpiBus, HostImpl> {
        self.into()
    }

    pub fn release_socket(&mut self, socket: Socket) {
        self.sockets.set_bit(socket.index.into(), true);
    }

    pub fn release(self) -> (SpiBus, HostImpl) {
        (self.bus, self.host)
    }
}

impl<Spi: FullDuplex<u8>, ChipSelect: OutputPin, HostImpl: Host>
    Device<ActiveFourWire<Spi, ChipSelect>, HostImpl>
{
    pub fn deactivate(self) -> (InactiveDevice<FourWire<ChipSelect>, HostImpl>, Spi) {
        let (bus, spi) = self.bus.deactivate();
        (InactiveDevice::new(bus, self.host), spi)
    }
}

impl<Spi: FullDuplex<u8>, HostImpl: Host> Device<ActiveThreeWire<Spi>, HostImpl> {
    pub fn deactivate(self) -> (InactiveDevice<ThreeWire, HostImpl>, Spi) {
        let (bus, spi) = self.bus.deactivate();
        (InactiveDevice::new(bus, self.host), spi)
    }
}
