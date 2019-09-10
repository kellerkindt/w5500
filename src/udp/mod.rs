mod inactive_udp_socket;
mod packet;

use byteorder::{BigEndian, ByteOrder};
use crate::IpAddress;
use crate::bus::ActiveBus;
use crate::network::Network;
use crate::socket::OwnedSockets;
use crate::socket::Socket;
use crate::udp::inactive_udp_socket::InactiveUdpSocket;
use crate::w5500::W5500;
use crate::udp::packet::UdpPacket;
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

    /// Returns a UDP packet if one is available.  Will return `None` if no UDP packets are in the socket's buffer
    pub fn receive(mut self) -> Result<Option<UdpPacket<Self>>, SpiBus::Error> {
        if !self.socket.has_received(&mut self.bus)? {
            Ok(None)
        } else {
            Ok(Some(UdpPacket::new(self)?))
        }
    }

    fn block_until_receive_size_known(&mut self) -> Result<u16, SpiBus::Error> {
        loop {
            let mut sample_0 = [0u8; 2];
            block!(self.bus.transfer_frame(self.socket.register(), socketn::RECEIVED_SIZE, false, &mut sample_0))?;
            let mut sample_1 = [0u8; 2];
            block!(self.bus.transfer_frame(self.socket.register(), socketn::RECEIVED_SIZE, false, &mut sample_1))?;
            if sample_0 == sample_1 && sample_0[0] >= 8 {
                break Ok(BigEndian::read_u16(&sample_0));
            }
        }
    }

//     /// Sends a UDP packet to the specified IP and port, and blocks until it is sent
//     fn blocking_send(
//         &mut self,
//         host: &IpAddress,
//         host_port: u16,
//         data: &[u8],
//     ) -> Result<(), Self::Error> {
//         let (w5500, UdpSocket(socket)) = self;

//         {
//             let local_port = w5500.read_u16(socket.at(SocketRegister::LocalPort))?;
//             let local_port = local_port.to_be_bytes();
//             let host_port = host_port.to_be_bytes();

//             w5500.write_to(
//                 socket.at(SocketRegister::LocalPort),
//                 &[
//                     local_port[0],
//                     local_port[1], // local port u16
//                     0x00,
//                     0x00,
//                     0x00,
//                     0x00,
//                     0x00,
//                     0x00, // destination mac
//                     host.address[0],
//                     host.address[1],
//                     host.address[2],
//                     host.address[3], // target IP
//                     host_port[0],
//                     host_port[1], // destination port (5354)
//                 ],
//             )?;
//         }

//         let data_length = data.len() as u16;
//         {
//             let data_length = data_length.to_be_bytes();

//             // TODO why write [0x00, 0x00] at TxReadPointer at all?
//             // TODO Is TxWritePointer not sufficient enough?
//             w5500.write_to(
//                 socket.at(SocketRegister::TxReadPointer),
//                 &[0x00, 0x00, data_length[0], data_length[1]],
//             );
//         }

//         w5500.write_to(
//             socket.tx_register_at(0x00_00),
//             &data[..data_length as usize],
//         )?;

//         w5500.write_to(
//             socket.at(SocketRegister::Command),
//             &[SocketCommand::Send as u8],
//         )?;

//         for _ in 0..0xFFFF {
//             // wait until sent
//             if w5500.is_interrupt_set(*socket, Interrupt::SendOk)? {
//                 w5500.reset_interrupt(*socket, Interrupt::SendOk)?;
//                 break;
//             }
//         }
//         // restore listen state
//         w5500.write_to(
//             socket.at(SocketRegister::Mode),
//             &[
//                 Protocol::UDP as u8,       // Socket Mode Register
//                 SocketCommand::Open as u8, // Socket Command Register
//             ],
//         )?;
//         Ok(())
//     }


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
