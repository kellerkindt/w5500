use embedded_hal::blocking::spi::{Transfer, Write};
use embedded_hal::digital::v2::OutputPin;
use embedded_nal::Ipv4Addr;

use crate::bus::{Bus, FourWire, ThreeWire};
use crate::device::Device;
use crate::host::{Dhcp, Host, Manual};
use crate::raw_device::RawDevice;
use crate::register;
use crate::{MacAddress, Mode};

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

    /// The gateway overrides the passed `ip` ([`Ip4Addr`]) to end with `.1`.
    ///
    /// E.g. `let ip = "192.168.0.201".parse::<Ip4Addr>()` will become a device with a gateway `192.168.0.1`.
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
        let mode = [0b10000000];
        self.bus
            .write_frame(register::COMMON, register::common::MODE, &mode)?;

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

    fn set_mode(&mut self, mode_options: Mode) -> Result<(), SpiBus::Error> {
        let mut mode = [0];
        mode[0] |= mode_options.on_wake_on_lan as u8;
        mode[0] |= mode_options.on_ping_request as u8;
        mode[0] |= mode_options.connection_type as u8;
        mode[0] |= mode_options.arp_responses as u8;
        self.bus
            .write_frame(register::COMMON, register::common::MODE, &mode)?;
        Ok(())
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
