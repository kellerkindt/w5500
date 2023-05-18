use bit_field::BitArray;
use embedded_hal::digital::v2::OutputPin;

use crate::bus::{Bus, BusRef, FourWire, SpiRef, ThreeWire};
use crate::host::Host;
use crate::net::Ipv4Addr;
use crate::socket::Socket;
use crate::uninitialized_device::UninitializedDevice;
use crate::{register, MacAddress};

#[derive(Debug, Clone)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum ResetError<E> {
    SocketsNotReleased,
    Other(E),
}

impl<E> From<E> for ResetError<E> {
    fn from(error: E) -> ResetError<E> {
        ResetError::Other(error)
    }
}

#[derive(Debug)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub struct DeviceState<HostImpl: Host> {
    host: HostImpl,
    sockets: [u8; 1],
}

pub struct Device<SpiBus: Bus, HostImpl: Host> {
    bus: SpiBus,
    state: DeviceState<HostImpl>,
}

impl<SpiBus: Bus, HostImpl: Host> Device<SpiBus, HostImpl> {
    pub(crate) fn new(bus: SpiBus, host: HostImpl) -> Self {
        Device {
            bus,
            state: DeviceState {
                host,
                sockets: [0b11111111],
            },
        }
    }

    pub fn get_state(&self) -> &DeviceState<HostImpl> {
        &self.state
    }

    pub fn reset(mut self) -> Result<UninitializedDevice<SpiBus>, ResetError<SpiBus::Error>> {
        if self.state.sockets != [0b11111111] {
            Err(ResetError::SocketsNotReleased)
        } else {
            self.clear_mode()?;
            Ok(UninitializedDevice::new(self.bus))
        }
    }

    fn clear_mode(&mut self) -> Result<(), SpiBus::Error> {
        // reset bit
        let reset_mode = register::common::Mode::Reset;

        self.bus.write_frame(
            register::COMMON,
            register::common::MODE,
            &[reset_mode.to_u8()],
        )?;
        Ok(())
    }

    #[inline]
    pub fn gateway(&mut self) -> Result<Ipv4Addr, SpiBus::Error> {
        self.as_mut().gateway()
    }

    #[inline]
    pub fn subnet_mask(&mut self) -> Result<Ipv4Addr, SpiBus::Error> {
        self.as_mut().subnet_mask()
    }

    #[inline]
    pub fn mac(&mut self) -> Result<MacAddress, SpiBus::Error> {
        self.as_mut().mac()
    }

    #[inline]
    pub fn ip(&mut self) -> Result<Ipv4Addr, SpiBus::Error> {
        self.as_mut().ip()
    }

    #[inline]
    pub fn phy_config(&mut self) -> Result<register::common::PhyConfig, SpiBus::Error> {
        self.as_mut().phy_config()
    }

    #[inline]
    pub fn version(&mut self) -> Result<u8, SpiBus::Error> {
        self.as_mut().version()
    }

    #[inline]
    pub(crate) fn as_mut(&mut self) -> DeviceRefMut<'_, BusRef<'_, SpiBus>, HostImpl> {
        DeviceRefMut {
            bus: BusRef(&mut self.bus),
            state: &mut self.state,
        }
    }

    pub fn release(self) -> (SpiBus, HostImpl) {
        (self.bus, self.state.host)
    }

    pub fn deactivate(self) -> (SpiBus, InactiveDevice<HostImpl>) {
        (self.bus, InactiveDevice(self.state))
    }
}

#[derive(Debug)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub struct InactiveDevice<HostImpl: Host>(DeviceState<HostImpl>);

impl<HostImpl: Host> InactiveDevice<HostImpl> {
    /// Activates the device by taking ownership
    pub fn activate<SpiBus: Bus>(self, bus: SpiBus) -> Device<SpiBus, HostImpl> {
        Device { bus, state: self.0 }
    }

    /// Activates the device by borrowing it
    pub fn activate_ref<SpiBus: Bus>(&mut self, bus: SpiBus) -> DeviceRefMut<SpiBus, HostImpl> {
        DeviceRefMut {
            bus,
            state: &mut self.0,
        }
    }
}

pub struct DeviceRefMut<'a, SpiBus: Bus, HostImpl: Host> {
    pub(crate) bus: SpiBus,
    state: &'a mut DeviceState<HostImpl>,
}

impl<SpiBus: Bus, HostImpl: Host> DeviceRefMut<'_, SpiBus, HostImpl> {
    pub fn take_socket(&mut self) -> Option<Socket> {
        // TODO maybe return Future that resolves when release_socket invoked
        for index in 0..8 {
            if self.state.sockets.get_bit(index) {
                self.state.sockets.set_bit(index, false);
                return Some(Socket::new(index as u8));
            }
        }
        None
    }

    pub fn release_socket(&mut self, socket: Socket) {
        self.state.sockets.set_bit(socket.index.into(), true);
    }

    // /// RTR (Retry Time-value Register) [R/W] [0x0019 – 0x001A] [0x07D0]
    // pub fn set_retry_timeout(
    //     &self,
    //     code: socketn::Interrupt,
    // ) -> Result<(), SpiBus::Error> {
    //     let data = [code as u8];
    //     self.bus.write_frame(self.register(), socketn::, &data)?;
    //     Ok(())
    // }

    pub fn gateway(&mut self) -> Result<Ipv4Addr, SpiBus::Error> {
        let mut octets = [0u8; 4];
        self.bus
            .read_frame(register::COMMON, register::common::GATEWAY, &mut octets)?;
        Ok(Ipv4Addr::from(octets))
    }

    pub fn subnet_mask(&mut self) -> Result<Ipv4Addr, SpiBus::Error> {
        let mut octets = [0u8; 4];
        self.bus
            .read_frame(register::COMMON, register::common::SUBNET_MASK, &mut octets)?;
        Ok(Ipv4Addr::from(octets))
    }

    pub fn mac(&mut self) -> Result<MacAddress, SpiBus::Error> {
        let mut mac = MacAddress::default();
        self.bus
            .read_frame(register::COMMON, register::common::MAC, &mut mac.octets)?;
        Ok(mac)
    }

    pub fn ip(&mut self) -> Result<Ipv4Addr, SpiBus::Error> {
        let mut octets = [0u8; 4];
        self.bus
            .read_frame(register::COMMON, register::common::IP, &mut octets)?;
        Ok(Ipv4Addr::from(octets))
    }

    pub fn phy_config(&mut self) -> Result<register::common::PhyConfig, SpiBus::Error> {
        let mut phy = [0u8];
        self.bus
            .read_frame(register::COMMON, register::common::PHY_CONFIG, &mut phy)?;
        Ok(phy[0].into())
    }

    pub fn version(&mut self) -> Result<u8, SpiBus::Error> {
        let mut version = [0u8];
        self.bus
            .read_frame(register::COMMON, register::common::VERSION, &mut version)?;
        Ok(version[0])
    }
}
