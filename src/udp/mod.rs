mod inactive_udp_socket;

use crate::bus::ActiveBus;
use crate::network::Network;
use crate::socket::Socket;
use crate::w5500::W5500;
use crate::udp::inactive_udp_socket::InactiveUdpSocket;
use crate::socket::OwnedSockets;

pub struct UdpSocket<'a, SpiBus: ActiveBus, NetworkImpl: Network, SocketImpl: Socket> {
    bus: SpiBus,
    network: NetworkImpl,
    sockets: OwnedSockets,

    socket: &'a mut SocketImpl,
}

impl<'a, SpiBus: ActiveBus, NetworkImpl: Network, SocketImpl: Socket> UdpSocket<'a, SpiBus, NetworkImpl, SocketImpl> {
    pub fn new(bus: SpiBus, network: NetworkImpl, sockets: OwnedSockets, socket: &'a mut SocketImpl) -> Self {
        // TODO setup socket in UDP mode
        // self.0.reset_interrupt(socket, Interrupt::SendOk)?;
        // self.0.write_u16(socket.at(SocketRegister::LocalPort), port)?;
        // self.0.write_to(
        //     socket.at(SocketRegister::Mode),
        //     &[
        //         Protocol::UDP as u8,       // Socket Mode Register
        //         SocketCommand::Open as u8, // Socket Command Register
        //     ],
        // )?;

        UdpSocket { bus, network, sockets, socket }
    }

    pub fn deactivate(self) -> (InactiveUdpSocket<'a, SocketImpl>, W5500<SpiBus, NetworkImpl>) {
        (InactiveUdpSocket::new(self.socket), W5500::new(self.bus, self.network, self.sockets))
    }
}
