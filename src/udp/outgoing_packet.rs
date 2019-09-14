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

impl<'a, SpiBus: ActiveBus, NetworkImpl: Network, SocketImpl: Socket>
    OutgoingPacket<UdpSocket<'a, SpiBus, NetworkImpl, SocketImpl>>
{
    pub fn new(
        mut udp_socket: UdpSocket<'a, SpiBus, NetworkImpl, SocketImpl>,
        host: IpAddress,
        port: u16,
    ) -> Result<Self, SpiBus::Error> {
        udp_socket
            .socket
            .set_destination_ip(&mut udp_socket.bus, host)?;
        udp_socket
            .socket
            .set_destination_port(&mut udp_socket.bus, port)?;
        udp_socket
            .socket
            .set_tx_read_pointer(&mut udp_socket.bus, 0)?;
        Ok(Self {
            udp_socket,
            data_length: 0,
        })
    }

    pub fn write(&mut self, mut data: &mut [u8]) -> Result<(), SpiBus::Error> {
        block!(self.udp_socket.bus.transfer_frame(
            self.udp_socket.socket.tx_buffer(),
            self.data_length,
            true,
            &mut data
        ))?;
        self.data_length += data.len() as u16;
        Ok(())
    }

    pub fn send(mut self) -> Result<UdpSocket<'a, SpiBus, NetworkImpl, SocketImpl>, SpiBus::Error> {
        self.udp_socket
            .socket
            .set_tx_write_pointer(&mut self.udp_socket.bus, self.data_length)?;
        self.udp_socket
            .socket
            .command(&mut self.udp_socket.bus, socketn::Command::Send)?;
        loop {
            // wait until send is complete
            if self
                .udp_socket
                .socket
                .has_interrupt(&mut self.udp_socket.bus, socketn::Interrupt::SendOk)?
            {
                self.udp_socket
                    .socket
                    .reset_interrupt(&mut self.udp_socket.bus, socketn::Interrupt::SendOk)?;
                break;
            }
        }
        self.udp_socket
            .socket
            .command(&mut self.udp_socket.bus, socketn::Command::Open)?;
        Ok(self.udp_socket)
    }
}
