mod inactive_udp_socket;
mod incoming_packet;
mod outgoing_packet;

use crate::bus::ActiveBus;
use crate::network::Network;
use crate::register::socketn;
use crate::socket::{OwnedSockets, Socket};
use crate::udp::inactive_udp_socket::InactiveUdpSocket;
use crate::udp::incoming_packet::IncomingPacket;
use crate::udp::outgoing_packet::OutgoingPacket;
use crate::w5500::W5500;
use crate::IpAddress;

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
        socket.command(&mut bus, socketn::Command::Open)?;

        Ok(UdpSocket {
            bus,
            network,
            sockets,
            socket,
        })
    }

    /// Returns a UDP packet if one is available.  Will return `None` if no UDP packets are in the socket's buffer
    pub fn receive(mut self) -> Result<Option<IncomingPacket<Self>>, SpiBus::Error> {
        if !self
            .socket
            .has_interrupt(&mut self.bus, socketn::Interrupt::Receive)?
        {
            Ok(None)
        } else {
            Ok(Some(IncomingPacket::new(self)?))
        }
    }

    /// Sends a UDP packet to the specified IP and port, and blocks until it is sent
    pub fn send(
        self,
        host: IpAddress,
        remote_port: u16,
    ) -> Result<OutgoingPacket<Self>, SpiBus::Error> {
        Ok(OutgoingPacket::new(self, host, remote_port)?)
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
