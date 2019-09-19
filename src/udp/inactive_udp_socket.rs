use crate::bus::ActiveBus;
use crate::network::Network;
use crate::socket::Socket;
use crate::udp::UdpSocket;
use crate::w5500::W5500;

pub struct InactiveUdpSocket<SocketImpl: Socket> {
    socket: SocketImpl,
}

impl<SocketImpl: Socket> InactiveUdpSocket<SocketImpl> {
    pub fn new(socket: SocketImpl) -> Self {
        InactiveUdpSocket { socket }
    }

    pub fn activate<SpiBus: ActiveBus, NetworkImpl: Network>(
        self,
        w5500: W5500<SpiBus, NetworkImpl>,
    ) -> UdpSocket<SpiBus, NetworkImpl, SocketImpl> {
        UdpSocket {
            w5500,
            socket: self.socket,
        }
    }
}
