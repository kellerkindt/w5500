use crate::bus::ActiveBus;
use crate::network::Network;
use crate::register::socketn;
use crate::socket::Socket;
use crate::udp::UdpSocket;
use crate::IpAddress;
use byteorder::{BigEndian, ByteOrder};

pub struct IncomingPacket<UdpSocket> {
    udp_socket: UdpSocket,
    address: IpAddress,
    remote_port: u16,
    read_pointer: u16,
    write_pointer: u16,
}

impl<'a, SpiBus: ActiveBus, NetworkImpl: Network, SocketImpl: Socket>
    IncomingPacket<UdpSocket<'a, SpiBus, NetworkImpl, SocketImpl>>
{
    pub fn new(
        mut udp_socket: UdpSocket<'a, SpiBus, NetworkImpl, SocketImpl>,
    ) -> Result<Self, SpiBus::Error> {
        let receive_size = udp_socket.socket.get_receive_size(&mut udp_socket.bus)?;

        // Packet frame, as described in W5200 docs sectino 5.2.2.1
        // |<-- read_pointer                                read_pointer + received_size -->|
        // |Destination IP Address | Destination Port | Byte Size of DATA | Actual DATA ... |
        // |   --- 4 Bytes ---     |  --- 2 Bytes --- |  --- 2 Bytes ---  |      ....       |
        let read_pointer = udp_socket.socket.get_rx_read_pointer(&mut udp_socket.bus)?;
        let mut header = [0u8; 8];
        block!(udp_socket.bus.transfer_frame(
            udp_socket.socket.rx_buffer(),
            read_pointer,
            false,
            &mut header
        ))?;
        Ok(Self {
            udp_socket,
            address: IpAddress::new(header[0], header[1], header[2], header[3]),
            remote_port: BigEndian::read_u16(&header[4..5]),
            read_pointer: read_pointer + 8,
            write_pointer: read_pointer + receive_size,
        })
    }

    pub fn get_remote_port(&self) -> u16 {
        self.remote_port
    }

    pub fn get_address(&self) -> IpAddress {
        self.address
    }

    // TODO add read_all method

    pub fn done(mut self) -> Result<UdpSocket<'a, SpiBus, NetworkImpl, SocketImpl>, SpiBus::Error> {
        self.udp_socket
            .socket
            .set_rx_read_pointer(&mut self.udp_socket.bus, self.write_pointer)?;
        self.udp_socket
            .socket
            .command(&mut self.udp_socket.bus, socketn::Command::Receive)?;
        Ok(self.udp_socket)
    }
}

impl<'a, SpiBus: ActiveBus, NetworkImpl: Network, SocketImpl: Socket> Iterator
    for IncomingPacket<UdpSocket<'a, SpiBus, NetworkImpl, SocketImpl>>
{
    type Item = Result<u8, SpiBus::Error>;
    fn next(&mut self) -> Option<Self::Item> {
        if self.read_pointer > self.write_pointer {
            return None;
        }
        let mut buffer = [0u8];
        let result = block!(self.udp_socket.bus.transfer_frame(
            self.udp_socket.socket.rx_buffer(),
            self.read_pointer,
            false,
            &mut buffer
        ));
        self.read_pointer += 1;
        // TODO handle looping back?
        match result {
            Ok(_) => Some(Ok(buffer[0])),
            Result::Err(error) => Some(Err(error)),
        }
    }
}
