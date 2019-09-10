use crate::udp::UdpSocket;
use crate::network::Network;
use crate::bus::ActiveBus;
use crate::socket::Socket;
use crate::IpAddress;
use byteorder::{BigEndian, ByteOrder};

pub struct UdpPacket<UdpSocket> {
    udp_socket: UdpSocket,
    address: IpAddress,
    remote_port: u16,
    size: u16,
    read_pointer: u16,
}

impl<'a, SpiBus: ActiveBus, NetworkImpl: Network, SocketImpl: Socket> UdpPacket<UdpSocket<'a, SpiBus, NetworkImpl, SocketImpl>>
{
    pub fn new(mut udp_socket: UdpSocket<'a, SpiBus, NetworkImpl, SocketImpl>) -> Result<Self, SpiBus::Error> {
        let receive_size = udp_socket.block_until_receive_size_known()? - 8;
        // |<-- read_pointer                                read_pointer + received_size -->|
        // |Destination IP Address | Destination Port | Byte Size of DATA | Actual DATA ... |
        // |   --- 4 Bytes ---     |  --- 2 Bytes --- |  --- 2 Bytes ---  |      ....       |
        let read_pointer = udp_socket.socket.get_packet_address(&mut udp_socket.bus)?;
        let mut header = [0u8; 8];
        block!(udp_socket.bus.transfer_frame(udp_socket.socket.rx_buffer(), read_pointer, false, &mut header))?;
        Ok(UdpPacket {
            udp_socket,
            address: IpAddress::new(header[0], header[1], header[2], header[3]),
            remote_port: BigEndian::read_u16(&header[4..5]),
            size: receive_size,
            read_pointer,
        })
    }

    pub fn get_remote_port(&self) -> u16 {
        self.remote_port
    }

    pub fn get_address(&self) -> IpAddress {
        self.address
    }

    pub fn read(&self) {
        // TODO read a byte upon every invocation
        // w5500.read_from(
        //     socket.rx_register_at(read_pointer + 8),
        //     &mut destination[..data_length],
        // )?;
    }

    pub fn close(self) -> UdpSocket<'a, SpiBus, NetworkImpl, SocketImpl> {
        // TODO reset read pointer etc

        // // reset
        // w5500.write_u16(
        //     socket.at(SocketRegister::RxReadPointer),
        //     read_pointer + receive_size as u16,
        // )?;
        // w5500.write_u8(
        //     socket.at(SocketRegister::Command),
        //     SocketCommand::Recv as u8,
        // )?;
        // TODO return Socket
        self.udp_socket
    }
}
