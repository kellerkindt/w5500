use crate::bus::Bus;
use crate::device::{Device, DeviceRefMut};
use crate::host::Host;
use crate::register::socketn;
use crate::socket::Socket;
use core::fmt::Debug;
use embedded_nal::{nb, IpAddr, Ipv4Addr, SocketAddr, SocketAddrV4, UdpClientStack, UdpFullStack};

pub struct UdpSocket {
    socket: Socket,
}

impl UdpSocket {
    fn new(socket: Socket) -> Self {
        UdpSocket { socket }
    }

    fn open<SpiBus: Bus>(
        &mut self,
        bus: &mut SpiBus,
        local_port: u16,
    ) -> Result<(), SpiBus::Error> {
        self.socket.command(bus, socketn::Command::Close)?;
        self.socket.reset_interrupt(bus, socketn::Interrupt::All)?;
        self.socket.set_source_port(bus, local_port)?;
        self.socket.set_mode(bus, socketn::Protocol::Udp)?;
        self.socket.set_interrupt_mask(
            bus,
            socketn::Interrupt::SendOk as u8 & socketn::Interrupt::Timeout as u8,
        )?;
        self.socket.command(bus, socketn::Command::Open)?;
        Ok(())
    }

    fn set_destination<SpiBus: Bus>(
        &mut self,
        bus: &mut SpiBus,
        remote: SocketAddrV4,
    ) -> Result<(), UdpSocketError<SpiBus::Error>> {
        self.socket.set_destination_ip(bus, *remote.ip())?;
        self.socket.set_destination_port(bus, remote.port())?;
        Ok(())
    }

    fn send<SpiBus: Bus>(
        &self,
        bus: &mut SpiBus,
        send_buffer: &[u8],
    ) -> NbResult<(), UdpSocketError<SpiBus::Error>> {
        // TODO increase longevity by cycling through buffer, instead of always writing to 0
        // TODO ensure write is currently possible
        self.socket
            .set_tx_read_pointer(bus, 0)
            .and_then(|_| bus.write_frame(self.socket.tx_buffer(), 0, send_buffer))
            .and_then(|_| {
                self.socket
                    .set_tx_write_pointer(bus, send_buffer.len() as u16)
            })
            .and_then(|_| self.socket.command(bus, socketn::Command::Send))?;

        loop {
            if self.socket.get_tx_read_pointer(bus)? == self.socket.get_tx_write_pointer(bus)? {
                if self.socket.has_interrupt(bus, socketn::Interrupt::SendOk)? {
                    self.socket
                        .reset_interrupt(bus, socketn::Interrupt::SendOk)?;
                    return Ok(());
                } else if self
                    .socket
                    .has_interrupt(bus, socketn::Interrupt::Timeout)?
                {
                    self.socket
                        .reset_interrupt(bus, socketn::Interrupt::Timeout)?;
                    return Err(NbError::Other(UdpSocketError::WriteTimeout));
                }
            }
        }
    }

    fn send_to<SpiBus: Bus>(
        &mut self,
        bus: &mut SpiBus,
        remote: SocketAddrV4,
        send_buffer: &[u8],
    ) -> NbResult<(), UdpSocketError<SpiBus::Error>> {
        self.set_destination(bus, remote)?;
        self.send(bus, send_buffer)
    }

    fn receive<SpiBus: Bus>(
        &mut self,
        bus: &mut SpiBus,
        receive_buffer: &mut [u8],
    ) -> NbResult<(usize, SocketAddr), UdpSocketError<SpiBus::Error>> {
        if !self
            .socket
            .has_interrupt(bus, socketn::Interrupt::Receive)?
        {
            return Err(NbError::WouldBlock);
        }

        /*
         * Packet frame, as described in W5200 docs section 5.2.2.1
         * |<-- read_pointer                                 read_pointer + received_size -->|
         * | Destination IP Address | Destination Port | Byte Size of DATA | Actual DATA ... |
         * |    --- 4 Bytes ---     |  --- 2 Bytes --- |  --- 2 Bytes ---  |      ....       |
         */
        // TODO loop until RX received size stops changing, or it's larger than
        // receive_buffer.len()
        let read_pointer = self.socket.get_rx_read_pointer(bus)?;
        let mut header = [0u8; 8];
        bus.read_frame(self.socket.rx_buffer(), read_pointer, &mut header)?;
        let remote = SocketAddr::new(
            IpAddr::V4(Ipv4Addr::new(header[0], header[1], header[2], header[3])),
            u16::from_be_bytes([header[4], header[5]]),
        );
        let packet_size = u16::from_be_bytes([header[6], header[7]]).into();
        let data_read_pointer = read_pointer + 8;
        // TODO handle buffer overflow
        bus.read_frame(
            self.socket.rx_buffer(),
            data_read_pointer,
            &mut receive_buffer[0..packet_size],
        )?;

        let tx_write_pointer = self.socket.get_tx_write_pointer(bus)?;
        self.socket
            .set_rx_read_pointer(bus, tx_write_pointer)
            .and_then(|_| self.socket.command(bus, socketn::Command::Receive))
            .and_then(|_| self.socket.command(bus, socketn::Command::Open))?;
        self.socket
            .reset_interrupt(bus, socketn::Interrupt::Receive)?;
        Ok((packet_size, remote))
    }

    fn close<SpiBus: Bus>(&self, bus: &mut SpiBus) -> Result<(), UdpSocketError<SpiBus::Error>> {
        self.socket.set_mode(bus, socketn::Protocol::Closed)?;
        self.socket.command(bus, socketn::Command::Close)?;
        Ok(())
    }
}

#[derive(Debug)]
pub enum UdpSocketError<E: Debug> {
    NoMoreSockets,
    UnsupportedAddress,
    Other(E),
    WriteTimeout,
}

impl<E: Debug> From<E> for UdpSocketError<E> {
    fn from(error: E) -> UdpSocketError<E> {
        UdpSocketError::Other(error)
    }
}

type NbResult<T, E> = Result<T, NbError<E>>;
enum NbError<E> {
    Other(E),
    WouldBlock,
}

impl<E: Debug> From<UdpSocketError<E>> for NbError<UdpSocketError<E>> {
    fn from(error: UdpSocketError<E>) -> NbError<UdpSocketError<E>> {
        NbError::Other(error)
    }
}

impl<E: Debug> From<E> for NbError<UdpSocketError<E>> {
    fn from(error: E) -> NbError<UdpSocketError<E>> {
        NbError::Other(UdpSocketError::Other(error))
    }
}

impl<E: Debug> From<NbError<E>> for nb::Error<E> {
    fn from(error: NbError<E>) -> nb::Error<E> {
        match error {
            NbError::Other(e) => nb::Error::Other(e),
            NbError::WouldBlock => nb::Error::WouldBlock,
        }
    }
}

impl<SpiBus, HostImpl> UdpClientStack for Device<SpiBus, HostImpl>
where
    SpiBus: Bus,
    HostImpl: Host,
{
    type UdpSocket = UdpSocket;
    type Error = UdpSocketError<SpiBus::Error>;

    #[inline]
    fn socket(&mut self) -> Result<Self::UdpSocket, Self::Error> {
        self.as_mut().socket()
    }

    #[inline]
    fn connect(
        &mut self,
        socket: &mut Self::UdpSocket,
        remote: SocketAddr,
    ) -> Result<(), Self::Error> {
        self.as_mut().connect(socket, remote)
    }

    #[inline]
    fn send(&mut self, socket: &mut Self::UdpSocket, buffer: &[u8]) -> nb::Result<(), Self::Error> {
        self.as_mut().send(socket, buffer)
    }

    #[inline]
    fn receive(
        &mut self,
        socket: &mut Self::UdpSocket,
        buffer: &mut [u8],
    ) -> nb::Result<(usize, SocketAddr), Self::Error> {
        self.as_mut().receive(socket, buffer)
    }

    #[inline]
    fn close(&mut self, socket: Self::UdpSocket) -> Result<(), Self::Error> {
        self.as_mut().close(socket)
    }
}

impl<SpiBus, HostImpl> UdpClientStack for DeviceRefMut<'_, SpiBus, HostImpl>
where
    SpiBus: Bus,
    HostImpl: Host,
{
    type UdpSocket = UdpSocket;
    type Error = UdpSocketError<SpiBus::Error>;

    fn socket(&mut self) -> Result<Self::UdpSocket, Self::Error> {
        if let Some(socket) = self.take_socket() {
            Ok(UdpSocket::new(socket))
        } else {
            Err(Self::Error::NoMoreSockets)
        }
    }

    fn connect(
        &mut self,
        socket: &mut Self::UdpSocket,
        remote: SocketAddr,
    ) -> Result<(), Self::Error> {
        if let SocketAddr::V4(remote) = remote {
            // TODO dynamically select a random port
            socket.open(&mut self.bus, 49849 + u16::from(socket.socket.index))?; // chosen by fair dice roll.
                                                                                 // guaranteed to be random.
            socket.set_destination(&mut self.bus, remote)?;
            Ok(())
        } else {
            Err(Self::Error::UnsupportedAddress)
        }
    }

    fn send(&mut self, socket: &mut Self::UdpSocket, buffer: &[u8]) -> nb::Result<(), Self::Error> {
        socket.send(&mut self.bus, buffer)?;
        Ok(())
    }

    fn receive(
        &mut self,
        socket: &mut Self::UdpSocket,
        buffer: &mut [u8],
    ) -> nb::Result<(usize, SocketAddr), Self::Error> {
        Ok(socket.receive(&mut self.bus, buffer)?)
    }

    fn close(&mut self, socket: Self::UdpSocket) -> Result<(), Self::Error> {
        socket.close(&mut self.bus)?;
        self.release_socket(socket.socket);
        Ok(())
    }
}

impl<SpiBus, HostImpl> UdpFullStack for Device<SpiBus, HostImpl>
where
    SpiBus: Bus,
    HostImpl: Host,
{
    #[inline]
    fn bind(&mut self, socket: &mut Self::UdpSocket, local_port: u16) -> Result<(), Self::Error> {
        self.as_mut().bind(socket, local_port)
    }

    #[inline]
    fn send_to(
        &mut self,
        socket: &mut Self::UdpSocket,
        remote: SocketAddr,
        buffer: &[u8],
    ) -> nb::Result<(), Self::Error> {
        self.as_mut().send_to(socket, remote, buffer)
    }
}

impl<SpiBus, HostImpl> UdpFullStack for DeviceRefMut<'_, SpiBus, HostImpl>
where
    SpiBus: Bus,
    HostImpl: Host,
{
    fn bind(&mut self, socket: &mut Self::UdpSocket, local_port: u16) -> Result<(), Self::Error> {
        socket.open(&mut self.bus, local_port)?;
        Ok(())
    }

    fn send_to(
        &mut self,
        socket: &mut Self::UdpSocket,
        remote: SocketAddr,
        buffer: &[u8],
    ) -> nb::Result<(), Self::Error> {
        if let SocketAddr::V4(remote) = remote {
            socket.send_to(&mut self.bus, remote, buffer)?;
            Ok(())
        } else {
            Err(nb::Error::Other(Self::Error::UnsupportedAddress))
        }
    }
}
