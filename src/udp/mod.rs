mod inactive_udp_socket;

use crate::bus::ActiveBus;
use crate::network::Network;
use crate::socket::OwnedSockets;
use crate::socket::Socket;
use crate::udp::inactive_udp_socket::InactiveUdpSocket;
use crate::w5500::W5500;
use register::socketn;

pub struct UdpSocket<'a, SpiBus: ActiveBus, NetworkImpl: Network, SocketImpl: Socket> {
    bus: SpiBus,
    network: NetworkImpl,
    sockets: OwnedSockets,

    socket: &'a mut SocketImpl,
}

impl<'a, SpiBus: ActiveBus, NetworkImpl: Network, SocketImpl: Socket>
    UdpSocket<'a, SpiBus, NetworkImpl, SocketImpl>
{
    pub fn new(
        port: u16,
        mut bus: SpiBus,
        network: NetworkImpl,
        sockets: OwnedSockets,
        socket: &'a mut SocketImpl,
    ) -> Result<Self, SpiBus::Error> {
        socket.reset_interrupt(&mut bus, socketn::Interrupt::SendOk)?;
        socket.set_source_port(&mut bus, port)?;
        socket.set_mode(&mut bus, socketn::Protocol::Udp)?;

        Ok(UdpSocket {
            bus,
            network,
            sockets,
            socket,
        })
    }

    pub fn deactivate(
        self,
    ) -> (
        InactiveUdpSocket<'a, SocketImpl>,
        W5500<SpiBus, NetworkImpl>,
    ) {
        (
            InactiveUdpSocket::new(self.socket),
            W5500::new(self.bus, self.network, self.sockets),
        )
    }
}
