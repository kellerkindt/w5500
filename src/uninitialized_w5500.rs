use crate::network::{Dhcp, Manual, Network};
use crate::{IpAddress, MacAddress, Mode};
use bus::{ActiveBus, ActiveFourWire, ActiveThreeWire};
use embedded_hal::digital::v2::OutputPin;
use embedded_hal::spi::FullDuplex;
use register;
use socket::OwnedSockets;
use socket::{Socket0, Socket1, Socket2, Socket3, Socket4, Socket5, Socket6, Socket7};
use w5500::W5500;

pub struct UninitializedW5500<SpiBus: ActiveBus> {
    bus: SpiBus,
}

#[repr(u8)]
pub enum InitializeError<SpiError> {
    SpiError(SpiError),
    ChipNotConnected,
}

impl<SpiBus: ActiveBus> UninitializedW5500<SpiBus> {
    pub fn new(bus: SpiBus) -> UninitializedW5500<SpiBus> {
        UninitializedW5500 { bus: bus }
    }

    pub fn initialize(
        self,
        mac: MacAddress,
        mode_options: Mode,
    ) -> Result<(W5500<SpiBus, Dhcp>, OwnedSockets), InitializeError<SpiBus::Error>> {
        let network = Dhcp::new(mac);
        self.initialize_with_network(network, mode_options)
    }

    pub fn initialize_manual(
        self,
        mac: MacAddress,
        ip: IpAddress,
        mode_options: Mode,
    ) -> Result<(W5500<SpiBus, Manual>, OwnedSockets), InitializeError<SpiBus::Error>> {
        let mut gateway = ip;
        gateway.address[3] = 1;
        let subnet = IpAddress::new(255, 255, 255, 0);
        self.initialize_advanced(mac, ip, gateway, subnet, mode_options)
    }

    pub fn initialize_advanced(
        self,
        mac: MacAddress,
        ip: IpAddress,
        gateway: IpAddress,
        subnet: IpAddress,
        mode_options: Mode,
    ) -> Result<(W5500<SpiBus, Manual>, OwnedSockets), InitializeError<SpiBus::Error>> {
        let network = Manual::new(mac, ip, gateway, subnet);
        self.initialize_with_network(network, mode_options)
    }

    fn initialize_with_network<NetworkImpl: Network>(
        mut self,
        mut network: NetworkImpl,
        mode_options: Mode,
    ) -> Result<(W5500<SpiBus, NetworkImpl>, OwnedSockets), InitializeError<SpiBus::Error>> {
        self.assert_chip_version(0x4)?;
        self.set_mode(mode_options)
            .map_err(|e| InitializeError::SpiError(e))?;
        network
            .refresh(&mut self.bus)
            .map_err(|e| InitializeError::SpiError(e))?;
        let sockets = (
            Socket0 {},
            Socket1 {},
            Socket2 {},
            Socket3 {},
            Socket4 {},
            Socket5 {},
            Socket6 {},
            Socket7 {},
        );
        Ok((W5500::new(self.bus, network), sockets))
    }

    fn assert_chip_version(
        &mut self,
        expected_version: u8,
    ) -> Result<(), InitializeError<SpiBus::Error>> {
        let mut version = [0];
        self.bus.transfer_frame(
            register::COMMON,
            register::common::VERSION,
            false,
            &mut version
        )
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
        self
            .bus
            .transfer_frame(register::COMMON, register::common::MODE, true, &mut mode)?;
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
