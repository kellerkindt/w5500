use embedded_hal::blocking::spi::{Transfer, Write};
use embedded_hal::digital::v2::OutputPin;
use embedded_nal::Ipv4Addr;

use crate::bus::{Bus, FourWire, ThreeWire};
use crate::device::Device;
use crate::host::{Dhcp, Host, Manual};
use crate::raw_device::RawDevice;
use crate::{
    register::{
        self,
        common::{RetryCount, RetryTime},
    },
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
    ) -> Result<Device<SpiBus, Dhcp>, InitializeError<SpiBus::Error>> {
        let host = Dhcp::new(mac);
        self.initialize_with_host(host, mode_options)
    }

    pub fn initialize_manual(
        self,
        mac: MacAddress,
        ip: Ipv4Addr,
        mode_options: Mode,
    ) -> Result<Device<SpiBus, Manual>, InitializeError<SpiBus::Error>> {
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
    ) -> Result<Device<SpiBus, Manual>, InitializeError<SpiBus::Error>> {
        let host = Manual::new(mac, ip, gateway, subnet);
        self.initialize_with_host(host, mode_options)
    }

    fn initialize_with_host<HostImpl: Host>(
        mut self,
        mut host: HostImpl,
        mode_options: Mode,
    ) -> Result<Device<SpiBus, HostImpl>, InitializeError<SpiBus::Error>> {
        #[cfg(not(feature = "no-chip-version-assertion"))]
        self.assert_chip_version(0x4)?;

        // RESET
        self.reset()?;

        self.set_mode(mode_options)?;
        host.refresh(&mut self.bus)?;
        Ok(Device::new(self.bus, host))
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

    /// Get the currently set Retry Time-value Register.
    ///
    /// RTR (Retry Time-value Register) [R/W] [0x0019 – 0x001A] [0x07D0]
    ///
    /// E.g. 4000 = 400ms
    #[inline]
    pub fn current_retry_timeout(&mut self) -> Result<RetryTime, SpiBus::Error> {
        self.bus.current_retry_timeout()
    }

    /// Set a new value for the Retry Time-value Register.
    ///
    /// RTR (Retry Time-value Register) [R/W] [0x0019 – 0x001A] [0x07D0]
    ///
    /// # Example
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
        self.bus.set_retry_timeout(retry_time_value)
    }

    /// Get the current Retry Count Register value.
    ///
    /// RCR (Retry Count Register) [R/W] [0x001B] [0x08]
    ///
    /// E.g. In case of errors it will retry for 7 times:
    /// `RCR = 0x0007`
    #[inline]
    pub fn current_retry_count(&mut self) -> Result<RetryCount, SpiBus::Error> {
        self.bus.current_retry_count()
    }

    /// Set a new value for the Retry Count register.
    ///
    /// RCR (Retry Count Register) [R/W] [0x001B] [0x08]
    #[inline]
    pub fn set_retry_count(&mut self, retry_count: RetryCount) -> Result<(), SpiBus::Error> {
        self.bus.set_retry_count(retry_count)
    }

    #[cfg(not(feature = "no-chip-version-assertion"))]
    fn assert_chip_version(
        &mut self,
        expected_version: u8,
    ) -> Result<(), InitializeError<SpiBus::Error>> {
        let mut version = [0];
        self.bus
            .read_frame(register::COMMON, register::common::VERSION, &mut version)?;
        if version[0] != expected_version {
            Err(InitializeError::ChipNotConnected)
        } else {
            Ok(())
        }
    }

    /// RESET
    fn reset(&mut self) -> Result<(), SpiBus::Error> {
        self.bus.reset()
    }

    fn set_mode(&mut self, mode_options: Mode) -> Result<(), SpiBus::Error> {
        self.bus.set_mode(mode_options.into())
    }
}

impl<Spi: Transfer<u8> + Write<u8>, ChipSelect: OutputPin>
    UninitializedDevice<FourWire<Spi, ChipSelect>>
{
    pub fn deactivate(self) -> (Spi, ChipSelect) {
        self.bus.release()
    }
}

impl<Spi: Transfer<u8> + Write<u8>> UninitializedDevice<ThreeWire<Spi>> {
    pub fn deactivate(self) -> Spi {
        self.bus.release()
    }
}
