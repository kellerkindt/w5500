use crate::bus::ActiveBus;
use crate::network::Network;
use crate::socket::Socket;
use crate::IpAddress;
use crate::udp::UdpSocket;

pub struct OutgoingPacket<UdpSocket> {
    udp_socket: UdpSocket
}

impl<'a, SpiBus: ActiveBus, NetworkImpl: Network, SocketImpl: Socket> OutgoingPacket<UdpSocket<'a, SpiBus, NetworkImpl, SocketImpl>> {

    pub fn new(udp_socket: UdpSocket<'a, SpiBus, NetworkImpl, SocketImpl>, host: IpAddress, port: u16) -> Result<Self, SpiBus::Error>  {
        Ok(Self { udp_socket })
//         let (w5500, UdpSocket(socket)) = self;

//         {

//             // TODO set up packet destination
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

//         // TODO set read/write pointers
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

//         // TODO write body to buffer
//         w5500.write_to(
//             socket.tx_register_at(0x00_00),
//             &data[..data_length as usize],
//         )?;

//         // TODO execute send command
//         w5500.write_to(
//             socket.at(SocketRegister::Command),
//             &[SocketCommand::Send as u8],
//         )?;

//         // TODO wait until send is complete
//         for _ in 0..0xFFFF {
//             // wait until sent
//             if w5500.is_interrupt_set(*socket, Interrupt::SendOk)? {
//                 w5500.reset_interrupt(*socket, Interrupt::SendOk)?;
//                 break;
//             }
//         }
//         // TODO listen for incoming sockets
//         w5500.write_to(
//             socket.at(SocketRegister::Mode),
//             &[
//                 Protocol::UDP as u8,       // Socket Mode Register
//                 SocketCommand::Open as u8, // Socket Command Register
//             ],
//         )?;
//         Ok(())
    }
}
