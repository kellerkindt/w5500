use bit_field::BitArray;

use crate::bus::{Bus, FourWire, ThreeWire};
use crate::host::Host;
use crate::net::Ipv4Addr;
use crate::socket::Socket;
use crate::uninitialized_device::UninitializedDevice;
use crate::{
    register::{self, common::RetryTime},
    MacAddress, Mode,
};

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
    pub(crate) bus: SpiBus,
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
            self.reset_device()?;
            Ok(UninitializedDevice::new(self.bus))
        }
    }

    pub fn release(self) -> (SpiBus, HostImpl) {
        (self.bus, self.state.host)
    }

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

    #[inline]
    pub fn reset_device(&mut self) -> Result<(), SpiBus::Error> {
        // Set RST common register of the w5500
        let mode = [0b10000000];
        self.bus
            .write_frame(register::COMMON, register::common::MODE, &mode)
    }

    #[inline]
    pub fn set_mode(&mut self, mode_options: Mode) -> Result<(), SpiBus::Error> {
        self.bus.write_frame(
            register::COMMON,
            register::common::MODE,
            &mode_options.to_register(),
        )
    }

    #[inline]
    pub fn version(&mut self) -> Result<u8, SpiBus::Error> {
        let mut version_register = [0_u8];
        self.bus.read_frame(
            register::COMMON,
            register::common::VERSION,
            &mut version_register,
        )?;

        Ok(version_register[0])
    }

    /// Set a new value for the Retry Time-value Register.
    ///
    /// RTR (Retry Time-value Register) [R/W] [0x0019 – 0x001A] [0x07D0]
    ///
    /// # Example
    ///
    /// ```
    /// use w5500::register::common::RetryTime;
    ///
    /// let default = RetryTime::from_millis(200);
    /// assert_eq!(RetryTime::default(), default);
    ///
    /// // E.g. 4000 (register) = 400ms
    /// let four_hundred_ms = RetryTime::from_millis(400);
    /// assert_eq!(four_hundred_ms.to_u16(), 4000);
    /// ```
    #[inline]
    pub fn set_retry_timeout(&mut self, retry_time_value: RetryTime) -> Result<(), SpiBus::Error> {
        self.bus.write_frame(
            register::COMMON,
            register::common::RETRY_TIME,
            &retry_time_value.to_register(),
        )?;

        Ok(())
    }

    /// Get the currently set Retry Time-value Register.
    ///
    /// RTR (Retry Time-value Register) [R/W] [0x0019 – 0x001A] [0x07D0]
    ///
    /// E.g. 4000 = 400ms
    #[inline]
    pub fn current_retry_timeout(&mut self) -> Result<RetryTime, SpiBus::Error> {
        let mut retry_time_register: [u8; 2] = [0, 0];
        self.bus.read_frame(
            register::COMMON,
            register::common::RETRY_TIME,
            &mut retry_time_register,
        )?;

        Ok(RetryTime::from_register(retry_time_register))
    }

    /// Set a new value for the Retry Count register.
    ///
    /// RCR (Retry Count Register) [R/W] [0x001B] [0x08]
    ///
    /// For more details check out the rest of the datasheet documentation on the Retry count.
    ///
    /// From datasheet:
    ///
    /// RCR configures the number of time of retransmission. When retransmission occurs
    /// as many as ‘RCR+1’, Timeout interrupt is issued (Sn_IR[TIMEOUT] = ‘1’).
    ///
    /// The timeout of W5500 can be configurable with RTR and RCR. W5500 has two kind
    /// timeout such as Address Resolution Protocol (ARP) and TCP retransmission.
    ///
    /// E.g. In case of errors it will retry for 7 times:
    /// `RCR = 0x0007`
    pub fn set_retry_count(&mut self, retry_count: u8) -> Result<(), SpiBus::Error> {
        self.bus.write_frame(
            register::COMMON,
            register::common::RETRY_COUNT,
            &[retry_count],
        )?;

        Ok(())
    }

    /// Get the current Retry Count value
    ///
    /// RCR (Retry Count Register) [R/W] [0x001B] [0x08]
    ///
    /// E.g. In case of errors it will retry for 7 times:
    /// `RCR = 0x0007`
    #[inline]
    pub fn current_retry_count(&mut self) -> Result<u8, SpiBus::Error> {
        let mut retry_count_register: [u8; 1] = [0];
        self.bus.read_frame(
            register::COMMON,
            register::common::RETRY_COUNT,
            &mut retry_count_register,
        )?;

        Ok(retry_count_register[0])
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
}
