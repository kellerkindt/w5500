use crate::socket::Socket;
use crate::w5500::ForeignSocketError;
use crate::udp::UdpSocket;
use crate::w5500::W5500;
use crate::bus::ActiveBus;
use crate::network::Network;

pub struct InactiveUdpSocket<'a, SocketImpl: Socket> {
    socket: &'a mut SocketImpl,
}

impl<'a, SocketImpl: Socket> InactiveUdpSocket<'a, SocketImpl> {
    pub fn new(socket: &'a mut SocketImpl) -> Self {
        InactiveUdpSocket { socket }
    }

    pub fn activate<SpiBus: ActiveBus, NetworkImpl: Network>(self, w5500: W5500<SpiBus, NetworkImpl>) -> Result<UdpSocket<'a, SpiBus, NetworkImpl, SocketImpl>, ForeignSocketError> {
        let (bus, network, sockets) = w5500.release();
        if self.socket.is_owned_by(sockets) {
            Ok(UdpSocket::new(bus, network, sockets, self.socket))
        } else {
            Err(ForeignSocketError {})
        }
    }
}
