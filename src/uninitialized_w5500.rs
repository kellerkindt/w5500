use crate::network::{Dhcp, Manual, Network};
use crate::{MacAddress, Mode};
use bus::{ActiveBus, ActiveFourWire, ActiveThreeWire};
use embedded_hal::digital::v2::OutputPin;
use embedded_hal::spi::FullDuplex;
use embedded_nal::Ipv4Addr;
use register;
use w5500::W5500;

pub struct UninitializedW5500<SpiBus: ActiveBus> {
    bus: SpiBus,
}

#[repr(u8)]
pub enum InitializeError<SpiError> {
    SpiError(SpiError),
    ChipNotConnected,
}
// TODO add From impl and remove map_errs

impl<SpiBus: ActiveBus> UninitializedW5500<SpiBus> {
    pub fn new(bus: SpiBus) -> UninitializedW5500<SpiBus> {
        UninitializedW5500 { bus: bus }
    }

    pub fn initialize(
        self,
        mac: MacAddress,
        mode_options: Mode,
    ) -> Result<W5500<SpiBus, Dhcp>, InitializeError<SpiBus::Error>> {
        let network = Dhcp::new(mac);
        self.initialize_with_network(network, mode_options)
    }

    pub fn initialize_manual(
        self,
        mac: MacAddress,
        ip: Ipv4Addr,
        mode_options: Mode,
    ) -> Result<W5500<SpiBus, Manual>, InitializeError<SpiBus::Error>> {
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
    ) -> Result<W5500<SpiBus, Manual>, InitializeError<SpiBus::Error>> {
        let network = Manual::new(mac, ip, gateway, subnet);
        self.initialize_with_network(network, mode_options)
    }

    fn initialize_with_network<NetworkImpl: Network>(
        mut self,
        mut network: NetworkImpl,
        mode_options: Mode,
    ) -> Result<W5500<SpiBus, NetworkImpl>, InitializeError<SpiBus::Error>> {
        self.assert_chip_version(0x4)?;

        // RESET
        let mut mode = [0b10000000];
        self.bus
            .write_frame(register::COMMON, register::common::MODE, &mut mode)
            .map_err(|e| InitializeError::SpiError(e))?;

        self.set_mode(mode_options)
            .map_err(|e| InitializeError::SpiError(e))?;
        network
            .refresh(&mut self.bus)
            .map_err(|e| InitializeError::SpiError(e))?;
        Ok(W5500::new(self.bus, network))
    }

    fn assert_chip_version(
        &mut self,
        expected_version: u8,
    ) -> Result<(), InitializeError<SpiBus::Error>> {
        let mut version = [0];
        self.bus
            .read_frame(register::COMMON, register::common::VERSION, &mut version)
            .map_err(|e| InitializeError::SpiError(e))?;
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
            .write_frame(register::COMMON, register::common::MODE, &mut mode)?;
        Ok(())
    }
}

impl<Spi: FullDuplex<u8>, ChipSelect: OutputPin>
    UninitializedW5500<ActiveFourWire<Spi, ChipSelect>>
{
    pub fn deactivate(self) -> (Spi, ChipSelect) {
        let (bus, spi) = self.bus.deactivate();
        (spi, bus.release())
    }
}

impl<Spi: FullDuplex<u8>> UninitializedW5500<ActiveThreeWire<Spi>> {
    pub fn deactivate(self) -> Spi {
        self.bus.deactivate().1
    }
}
