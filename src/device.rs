use bit_field::BitArray;
use embedded_hal::digital::v2::OutputPin;

use crate::bus::{Bus, Fdm, Vdm};
use crate::host::Host;
use crate::register;
use crate::socket::Socket;
use crate::uninitialized_device::UninitializedDevice;

pub struct Device<SpiBus: Bus, HostImpl: Host> {
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

impl<SpiBus: Bus, HostImpl: Host> Device<SpiBus, HostImpl> {
    pub(crate) fn new(bus: SpiBus, host: HostImpl) -> Self {
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

    pub(crate) fn take_socket(&mut self) -> Option<Socket> {
        // TODO maybe return Future that resolves when release_socket invoked
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

    pub(crate) fn release_socket(&mut self, socket: Socket) {
        self.sockets.set_bit(socket.index.into(), true);
    }

    pub fn release(self) -> (SpiBus, HostImpl) {
        (self.bus, self.host)
    }

    pub fn deactivate(self) -> (SpiBus, InactiveDevice<HostImpl>) {
        (
            self.bus,
            InactiveDevice {
                host: self.host,
                sockets: self.sockets,
            },
        )
    }
}

pub struct InactiveDevice<HostImpl: Host> {
    host: HostImpl,
    sockets: [u8; 1],
}

impl<HostImpl: Host> InactiveDevice<HostImpl> {
    pub fn activate<SpiBus: Bus>(self, bus: SpiBus) -> Device<SpiBus, HostImpl> {
        Device {
            bus,
            host: self.host,
            sockets: self.sockets,
        }
    }
}
