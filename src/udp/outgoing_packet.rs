use crate::bus::ActiveBus;
use crate::network::Network;
use crate::register::socketn;
use crate::socket::Socket;
use crate::udp::UdpSocket;
use crate::IpAddress;

pub struct OutgoingPacket<UdpSocket> {
    udp_socket: UdpSocket,
    data_length: u16,
}

impl<SpiBus: ActiveBus, NetworkImpl: Network, SocketImpl: Socket>
    OutgoingPacket<UdpSocket<SpiBus, NetworkImpl, SocketImpl>>
{
    pub fn new(
        mut udp_socket: UdpSocket<SpiBus, NetworkImpl, SocketImpl>,
        host: IpAddress,
        port: u16,
    ) -> Result<Self, SpiBus::Error> {
        udp_socket
            .socket
            .set_destination_ip(&mut udp_socket.w5500.bus, host)?;
        udp_socket
            .socket
            .set_destination_port(&mut udp_socket.w5500.bus, port)?;
        udp_socket
            .socket
            .set_tx_read_pointer(&mut udp_socket.w5500.bus, 0)?;
        Ok(Self {
            udp_socket,
            data_length: 0,
        })
    }

    pub fn write(&mut self, mut data: &mut [u8]) -> Result<(), SpiBus::Error> {
        self.udp_socket.w5500.bus.transfer_frame(
            self.udp_socket.socket.tx_buffer(),
            self.data_length,
            true,
            &mut data
        )?;
        self.data_length += data.len() as u16;
        Ok(())
    }

    pub fn send(mut self) -> Result<UdpSocket<SpiBus, NetworkImpl, SocketImpl>, SpiBus::Error> {
        self.udp_socket
            .socket
            .set_tx_write_pointer(&mut self.udp_socket.w5500.bus, self.data_length)?;
        self.udp_socket
            .socket
            .command(&mut self.udp_socket.w5500.bus, socketn::Command::Send)?;
        loop {
            // wait until send is complete
            if self
                .udp_socket
                .socket
                .has_interrupt(&mut self.udp_socket.w5500.bus, socketn::Interrupt::SendOk)?
            {
                self.udp_socket
                    .socket
                    .reset_interrupt(&mut self.udp_socket.w5500.bus, socketn::Interrupt::SendOk)?;
                break;
            }
        }
        self.udp_socket
            .socket
            .command(&mut self.udp_socket.w5500.bus, socketn::Command::Open)?;
        Ok(self.udp_socket)
    }
}
