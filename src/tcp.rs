use crate::{
    bus::Bus,
    device::{Device, DeviceRefMut},
    host::Host,
    register::socketn,
    socket::Socket,
};

use embedded_nal::{nb, IpAddr, Ipv4Addr, SocketAddr, SocketAddrV4, TcpClientStack};

use core::convert::TryFrom;

#[derive(Debug)]
pub enum TcpSocketError<E: core::fmt::Debug> {
    NoMoreSockets,
    NotReady,
    UnsupportedAddress,
    Other(E),
    UnsupportedMode,
}

impl<E: core::fmt::Debug> From<E> for TcpSocketError<E> {
    fn from(e: E) -> Self {
        TcpSocketError::Other(e)
    }
}

pub struct TcpSocket{
    socket: Socket
}

impl TcpSocket {
    fn reopen<B: Bus>(&mut self, bus: &mut B) -> Result<(), TcpSocketError<B::Error>> {
        self.socket.command(bus, socketn::Command::Close)?;
        self.socket.reset_interrupt(bus, socketn::Interrupt::All)?;
        self.socket.set_mode(bus, socketn::Protocol::Tcp)?;

        self.socket.set_interrupt_mask(
            bus,
            socketn::Interrupt::SendOk as u8 & socketn::Interrupt::Timeout as u8,
        )?;

        self.socket.command(bus, socketn::Command::Open)?;
        Ok(())
    }
    fn open<B: Bus>(&mut self, bus: &mut B, local_port: u16) -> Result<(), TcpSocketError<B::Error>> {
        self.socket.command(bus, socketn::Command::Close)?;
        self.socket.reset_interrupt(bus, socketn::Interrupt::All)?;
        self.socket.set_source_port(bus, local_port)?;
        self.socket.set_mode(bus, socketn::Protocol::Tcp)?;

        self.socket.set_interrupt_mask(
            bus,
            socketn::Interrupt::SendOk as u8 & socketn::Interrupt::Timeout as u8,
        )?;

        self.socket.command(bus, socketn::Command::Open)?;
        Ok(())
    }

    fn close<B: Bus>(&self, bus: &mut B) -> Result<(), TcpSocketError<B::Error>> {
        self.socket.set_mode(bus, socketn::Protocol::Closed)?;
        self.socket.command(bus, socketn::Command::Close)?;
        Ok(())
    }

    fn connect<B: Bus>(&mut self, bus: &mut B, remote: SocketAddrV4) -> Result<(), TcpSocketError<B::Error>> {
        // Ensure the socket is open and ready before we attempt to connect it.
        match socketn::Status::try_from(self.socket.get_status(bus)?) {
            // Happy case: Nothing to do.
            Ok(socketn::Status::Init) => {},

            // If the socket is in the wrong mode, we can't use it. The user needs to re-open it.
            Err(_) | Ok(socketn::Status::MacRaw) | Ok(socketn::Status::Udp) => {
                return Err(TcpSocketError::UnsupportedMode)
            },

            // All other cases are transient TCP states. For these, we need to reset the TCP
            // machinery to return to the INIT state.
            Ok(_) => {
                self.close(bus)?;
                self.reopen(bus)?;
            }
        }

        // Write the remote port and IP
        self.socket.set_destination_ip(bus, *remote.ip())?;
        self.socket.set_destination_port(bus, remote.port())?;

        // Connect the socket.
        self.socket.command(bus, socketn::Command::Connect)?;

        // Wait for the socket to connect or encounter an error.
        loop {
            match socketn::Status::try_from(self.socket.get_status(bus)?) {
                Ok(socketn::Status::Established) => return Ok(()),

                // The socket is closed if a timeout (ARP or SYN-ACK) or if the TCP socket receives
                // a RST packet. In this case, the client will need to re-attempt to connect.

                // TODO: Due to limitations of the embedded-nal, we currently still return the
                // socket (since we cannot inform the user of the connection failure). The returned
                // socket will not actually be connected.
                Ok(socketn::Status::Closed) => {
                    // For now, always return an open socket so that the user can re-connect with
                    // it in the future.
                    self.close(bus)?;
                    return self.reopen(bus);
                }

                // The socket is still in some transient state. Wait for it to connect or for the
                // connection to fail.
                _ => {}
            }
        }
    }

    pub fn is_connected<B: Bus>(&self, bus: &mut B) -> Result<bool, TcpSocketError<B::Error>> {
        Ok(self.socket.get_status(bus)? == socketn::Status::Established as u8)
    }

    fn send<B: Bus>(&mut self, bus: &mut B, data: &[u8]) -> Result<usize, TcpSocketError<B::Error>> {
        if self.is_connected(bus)? == false {
            return Err(TcpSocketError::NotReady);
        }

        let max_size = self.socket.get_tx_free_size(bus)? as usize;

        let write_data = if data.len() < max_size {
            data
        } else {
            &data[..max_size]
        };

        // Append the data to the write buffer after the current write pointer.
        let write_pointer = self.socket.get_tx_write_pointer(bus)?;

        // Write data into the buffer and update the writer pointer.
        bus.write_frame(self.socket.tx_buffer(), write_pointer, &data)?;
        self.socket.set_tx_write_pointer(bus,
            write_pointer.wrapping_add(write_data.len() as u16))?;

        // Send the data.
        self.socket.command(bus, socketn::Command::Send)?;

        // Wait until the send command completes.
        while self.socket.has_interrupt(bus, socketn::Interrupt::SendOk)? == false {}
        self.socket.reset_interrupt(bus, socketn::Interrupt::SendOk)?;

        Ok(write_data.len())
    }

    fn receive<B: Bus>(&mut self, bus: &mut B, data: &mut [u8]) -> Result<usize, TcpSocketError<B::Error>> {
        if self.is_connected(bus)? == false {
            return Err(TcpSocketError::NotReady);
        }

        // Check if we've received data.
        if self.socket.has_interrupt(bus, socketn::Interrupt::Receive)? == false {
            return Ok(0);
        }

        let rx_size = self.socket.get_receive_size(bus)? as usize;

        let read_buffer = if rx_size > data.len() {
            data
        } else {
            &mut data[..rx_size]
        };

        // Read from the RX ring buffer.
        let read_pointer = self.socket.get_rx_read_pointer(bus)?;
        bus.read_frame(self.socket.rx_buffer(), read_pointer, read_buffer)?;
        self.socket.set_rx_read_pointer(bus, read_pointer.wrapping_add(read_buffer.len() as u16))?;

        // Register the reception as complete.
        self.socket.command(bus, socketn::Command::Receive)?;
        self.socket.reset_interrupt(bus, socketn::Interrupt::Receive)?;

        Ok(read_buffer.len())
    }
}

impl <SpiBus: Bus, HostImpl: Host> TcpClientStack for DeviceRefMut<'_, SpiBus, HostImpl> {
    type TcpSocket = TcpSocket;
    type Error = TcpSocketError<SpiBus::Error>;

    fn socket(&mut self) -> Result<TcpSocket, Self::Error> {
        match self.take_socket() {
            Some(socket) => Ok(TcpSocket {socket }),
            None => return Err(TcpSocketError::NoMoreSockets),
        }
    }

    fn connect(&mut self,
        socket: &mut Self::TcpSocket,
        remote: SocketAddr
    ) -> nb::Result<(), Self::Error> {
        if let SocketAddr::V4(remote) = remote {
            // TODO dynamically select a random port
            socket.open(&mut self.bus, 49849 + u16::from(socket.socket.index))?; // chosen by fair dice roll.
            socket.connect(&mut self.bus, remote)?;

            Ok(())
        } else {
            Err(nb::Error::Other(Self::Error::UnsupportedAddress))
        }
    }

    fn is_connected(
        &mut self,
        socket: &Self::TcpSocket,
    ) -> Result<bool, Self::Error> {
        Ok(socket.is_connected(&mut self.bus)?)
    }

    fn send(
        &mut self,
        socket: &mut Self::TcpSocket,
        buffer: &[u8],
    ) -> nb::Result<usize, Self::Error> {
        let len = socket.send(&mut self.bus, buffer)?;
        Ok(len)
    }

    fn receive(
        &mut self,
        socket: &mut Self::TcpSocket,
        buffer: &mut [u8],
    ) -> nb::Result<usize, Self::Error> {
        Ok(socket.receive(&mut self.bus, buffer)?)
    }

    fn close(&mut self, socket: Self::TcpSocket) -> Result<(), Self::Error> {
        socket.close(&mut self.bus)?;
        self.release_socket(socket.socket);
        Ok(())
    }
}

impl <SpiBus: Bus, HostImpl: Host> TcpClientStack for Device<SpiBus, HostImpl> {
    type TcpSocket = TcpSocket;
    type Error = TcpSocketError<SpiBus::Error>;

    fn socket(&mut self) -> Result<TcpSocket, Self::Error> {
        self.as_mut().socket()
    }

    fn connect(&mut self,
        socket: &mut Self::TcpSocket,
        remote: SocketAddr
    ) -> nb::Result<(), Self::Error> {
        self.as_mut().connect(socket, remote)
    }

    fn is_connected(
        &mut self,
        socket: &Self::TcpSocket,
    ) -> Result<bool, Self::Error> {
        self.as_mut().is_connected(socket)
    }

    fn send(
        &mut self,
        socket: &mut Self::TcpSocket,
        buffer: &[u8],
    ) -> nb::Result<usize, Self::Error> {
        self.as_mut().send(socket, buffer)
    }

    fn receive(
        &mut self,
        socket: &mut Self::TcpSocket,
        buffer: &mut [u8],
    ) -> nb::Result<usize, Self::Error> {
        self.as_mut().receive(socket, buffer)
    }

    fn close(&mut self, socket: Self::TcpSocket) -> Result<(), Self::Error> {
        self.as_mut().close(socket)
    }
}
