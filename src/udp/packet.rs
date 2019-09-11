use byteorder::{BigEndian, ByteOrder};
use crate::udp::UdpSocket;
use crate::network::Network;
use crate::bus::ActiveBus;
use crate::socket::Socket;
use crate::IpAddress;
use crate::register::socketn;

pub struct UdpPacket<UdpSocket> {
    udp_socket: UdpSocket,
    address: IpAddress,
    remote_port: u16,
    read_pointer: u16,
    write_pointer: u16,
}

impl<'a, SpiBus: ActiveBus, NetworkImpl: Network, SocketImpl: Socket> UdpPacket<UdpSocket<'a, SpiBus, NetworkImpl, SocketImpl>>
{
    pub fn new(mut udp_socket: UdpSocket<'a, SpiBus, NetworkImpl, SocketImpl>) -> Result<Self, SpiBus::Error> {
        let receive_size = Self::block_until_receive_size_known(&mut udp_socket)?;
        // |<-- read_pointer                                read_pointer + received_size -->|
        // |Destination IP Address | Destination Port | Byte Size of DATA | Actual DATA ... |
        // |   --- 4 Bytes ---     |  --- 2 Bytes --- |  --- 2 Bytes ---  |      ....       |
        let read_pointer = udp_socket.socket.get_rx_read_pointer(&mut udp_socket.bus)?;
        let mut header = [0u8; 8];
        block!(udp_socket.bus.transfer_frame(udp_socket.socket.rx_buffer(), read_pointer, false, &mut header))?;
        Ok(UdpPacket {
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

    pub fn read(&mut self) -> Result<u8, SpiBus::Error> {
        let mut buffer = [0u8];
        block!(self.udp_socket.bus.transfer_frame(self.udp_socket.socket.rx_buffer(), self.read_pointer, false, &mut buffer))?;
        Ok(buffer[0])
    }

    pub fn done(mut self) -> Result<UdpSocket<'a, SpiBus, NetworkImpl, SocketImpl>, SpiBus::Error> {
        self.udp_socket.socket.set_rx_read_pointer(&mut self.udp_socket.bus, self.write_pointer)?;
        self.udp_socket.socket.command(&mut self.udp_socket.bus, socketn::Command::Receive)?;
        Ok(self.udp_socket)
    }

    fn block_until_receive_size_known(udp_socket: &mut UdpSocket<'a, SpiBus, NetworkImpl, SocketImpl>) -> Result<u16, SpiBus::Error> {
        loop {
            let mut sample_0 = [0u8; 2];
            block!(udp_socket.bus.transfer_frame(udp_socket.socket.register(), socketn::RECEIVED_SIZE, false, &mut sample_0))?;
            let mut sample_1 = [0u8; 2];
            block!(udp_socket.bus.transfer_frame(udp_socket.socket.register(), socketn::RECEIVED_SIZE, false, &mut sample_1))?;
            if sample_0 == sample_1 && sample_0[0] >= 8 {
                break Ok(BigEndian::read_u16(&sample_0));
            }
        }
    }
}

impl<'a, SpiBus: ActiveBus, NetworkImpl: Network, SocketImpl: Socket> Iterator for UdpPacket<UdpSocket<'a, SpiBus, NetworkImpl, SocketImpl>> {
    type Item = Result<u8, SpiBus::Error>;
    fn next(&mut self) -> Option<Self::Item> {
        if self.read_pointer > self.write_pointer {
            return None;
        }
        let mut buffer = [0u8];
        let result = block!(self.udp_socket.bus.transfer_frame(self.udp_socket.socket.rx_buffer(), self.read_pointer, false, &mut buffer));
        self.read_pointer += 1;
        match result {
            Ok(_) => Some(Ok(buffer[0])),
            Result::Err(error) => Some(Err(error)),
        }
    }
}
