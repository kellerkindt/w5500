use core::net::Ipv4Addr;

use embedded_hal::spi::SpiDevice;

use crate::bus::{Bus, FourWire, ThreeWire};
use crate::device::{Device, DeviceState};
use crate::host::{Dhcp, Host, Manual};
use crate::raw_device::RawDevice;
use crate::{
    register::{self, common::RetryTime},
    MacAddress, Mode,
};

#[derive(Debug)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub struct UninitializedDevice<SpiBus: Bus> {
    bus: SpiBus,
}

#[derive(Debug)]
#[repr(u8)]
pub enum InitializeError<SpiError> {
    SpiError(SpiError),
    ChipNotConnected,
}

impl<SpiError> From<SpiError> for InitializeError<SpiError> {
    fn from(error: SpiError) -> InitializeError<SpiError> {
        InitializeError::SpiError(error)
    }
}

impl<SpiBus: Bus> UninitializedDevice<SpiBus> {
    pub fn new(bus: SpiBus) -> UninitializedDevice<SpiBus> {
        UninitializedDevice { bus }
    }

    /// Initialize the device with a MAC address and mode settings.
    ///
    /// Consider using freely available private/locally administered mac
    /// addresses that match the following hex pattern:
    ///
    /// ```code
    ///  x2-xx-xx-xx-xx-xx
    ///  x6-xx-xx-xx-xx-xx
    ///  xA-xx-xx-xx-xx-xx
    ///  xE-xx-xx-xx-xx-xx
    /// ```
    ///
    /// "Universally administered and locally administered addresses are
    /// distinguished by setting the second-least-significant bit of the first
    /// octet of the address"
    /// [Wikipedia](https://en.wikipedia.org/wiki/MAC_address#Universal_vs._local)
    pub fn initialize(
        self,
        mac: MacAddress,
        mode_options: Mode,
    ) -> Result<Device<SpiBus, DeviceState<Dhcp>>, InitializeError<SpiBus::Error>> {
        let host = Dhcp::new(mac);
        self.initialize_with_host(host, mode_options)
    }

    /// The gateway overrides the passed `ip` ([`Ip4Addr`]) to end with `.1`.
    ///
    /// E.g. `let ip = "192.168.0.201".parse::<Ip4Addr>()` will become a device with a gateway `192.168.0.1`.
    pub fn initialize_manual(
        self,
        mac: MacAddress,
        ip: Ipv4Addr,
        mode_options: Mode,
    ) -> Result<Device<SpiBus, DeviceState<Manual>>, InitializeError<SpiBus::Error>> {
        let mut ip_bytes = ip.octets();
        ip_bytes[3] = 1;
        let gateway = Ipv4Addr::from(ip_bytes);
        let subnet = Ipv4Addr::new(255, 255, 255, 0);
        self.initialize_advanced(mac, ip, gateway, subnet, mode_options)
    }

    pub fn initialize_advanced(
        self,
        mac: MacAddress,
        ip: Ipv4Addr,
        gateway: Ipv4Addr,
        subnet: Ipv4Addr,
        mode_options: Mode,
    ) -> Result<Device<SpiBus, DeviceState<Manual>>, InitializeError<SpiBus::Error>> {
        let host = Manual::new(mac, ip, gateway, subnet);
        self.initialize_with_host(host, mode_options)
    }

    fn initialize_with_host<HostImpl: Host>(
        mut self,
        mut host: HostImpl,
        mode_options: Mode,
    ) -> Result<Device<SpiBus, DeviceState<HostImpl>>, InitializeError<SpiBus::Error>> {
        #[cfg(not(feature = "no-chip-version-assertion"))]
        self.assert_chip_version(0x4)?;

        // RESET
        self.reset()?;

        self.set_mode(mode_options)?;
        host.refresh(&mut self.bus)?;
        Ok(Device::new(self.bus, DeviceState::new(host)))
    }

    pub fn initialize_macraw(
        mut self,
        mac: MacAddress,
    ) -> Result<RawDevice<SpiBus>, InitializeError<SpiBus::Error>> {
        // Reset the device.
        self.bus
            .write_frame(register::COMMON, register::common::MODE, &[0x80])?;

        self.bus
            .write_frame(register::COMMON, register::common::MAC, &mac.octets)?;

        RawDevice::new(self.bus)
    }

    /// Reset the device
    #[inline]
    pub fn reset(&mut self) -> Result<(), SpiBus::Error> {
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

    #[cfg(not(feature = "no-chip-version-assertion"))]
    fn assert_chip_version(
        &mut self,
        expected_version: u8,
    ) -> Result<(), InitializeError<SpiBus::Error>> {
        let version = self.version()?;

        if version != expected_version {
            Err(InitializeError::ChipNotConnected)
        } else {
            Ok(())
        }
    }
}
