use crate::inactive_w5500::InactiveW5500;
use crate::uninitialized_w5500::UninitializedW5500;
use bus::{ActiveBus, ActiveFourWire, ActiveThreeWire, FourWire, ThreeWire};
use embedded_hal::digital::v2::OutputPin;
use embedded_hal::spi::FullDuplex;
use network::Network;
use register;
use socket::{OwnedSockets, Socket, Sockets};
use udp::UdpSocket;

pub struct W5500<SpiBus: ActiveBus, NetworkImpl: Network> {
    bus: SpiBus,
    network: NetworkImpl,
    sockets: OwnedSockets,
}

impl<SpiBus: ActiveBus, NetworkImpl: Network> W5500<SpiBus, NetworkImpl> {
    pub fn new(bus: SpiBus, network: NetworkImpl, sockets: OwnedSockets) -> Self {
        W5500 {
            bus,
            network,
            sockets,
        }
    }

    pub fn sockets(&mut self) -> Sockets {
        (
            &mut self.sockets.0,
            &mut self.sockets.1,
            &mut self.sockets.2,
            &mut self.sockets.3,
            &mut self.sockets.4,
            &mut self.sockets.5,
            &mut self.sockets.6,
            &mut self.sockets.7,
        )
    }

    pub fn reset(mut self) -> Result<UninitializedW5500<SpiBus>, SpiBus::Error> {
        self.clear_mode()?;
        Ok(UninitializedW5500::new(self.bus))
    }

    fn clear_mode(&mut self) -> Result<(), SpiBus::Error> {
        // reset bit
        let mut mode = [0b10000000];
        block!(self
            .bus
            .transfer_frame(register::COMMON, register::common::MODE, true, &mut mode))?;
        Ok(())
    }

    pub fn open_udp_socket<SocketImpl: Socket>(&self, socket: SocketImpl) -> UdpSocket<SocketImpl> {
        // TODO compare socket to sockets list
        UdpSocket::new(socket)
    }
}

impl<Spi: FullDuplex<u8>, ChipSelect: OutputPin, NetworkImpl: Network>
    W5500<ActiveFourWire<Spi, ChipSelect>, NetworkImpl>
{
    pub fn deactivate(self) -> (InactiveW5500<FourWire<ChipSelect>, NetworkImpl>, Spi) {
        let (bus, spi) = self.bus.deactivate();
        (InactiveW5500::new(bus, self.network, self.sockets), spi)
    }
}

impl<Spi: FullDuplex<u8>, NetworkImpl: Network> W5500<ActiveThreeWire<Spi>, NetworkImpl> {
    pub fn deactivate(self) -> (InactiveW5500<ThreeWire, NetworkImpl>, Spi) {
        let (bus, spi) = self.bus.deactivate();
        (InactiveW5500::new(bus, self.network, self.sockets), spi)
    }
}
