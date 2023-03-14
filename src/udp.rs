use crate::bus::Bus;
use crate::device::{Device, DeviceRefMut};
use crate::host::Host;
use crate::register::socketn;
use crate::socket::Socket;
use core::fmt::Debug;
use embedded_nal::{nb, IpAddr, Ipv4Addr, SocketAddr, SocketAddrV4, UdpClientStack, UdpFullStack};

/// W5500 UDP Header
pub struct UdpHeader {
    /// The origin socket address (IP address and port).
    pub origin: SocketAddrV4,
    /// Length of the UDP packet in bytes.
    ///
    /// This may not be equal to the length of the data in the socket buffer if the UDP packet was truncated
    /// due to small buffer or the size of the internal W5500 RX buffer for the socket.
    pub len: usize,
}

#[derive(Debug)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
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

    fn send_all<SpiBus: Bus>(
        &self,
        bus: &mut SpiBus,
        send_buffer: &[u8],
    ) -> NbResult<(), UdpSocketError<SpiBus::Error>> {
        let mut free_size = self.socket.get_tx_free_size(bus)?;

        // ensure write is currently possible
        if free_size == 0 {
            return Err(NbError::WouldBlock);
        }

        // on the first send, we rely on the free TX size and the length of the send_buffer
        let mut total_sent = 0;

        // if we've sent all bytes, exit the loop
        while total_sent != send_buffer.len() {
            let remaining_bytes = &send_buffer[total_sent..];

            // take the smaller value out of the free buffer length and
            // the length of the remaining bytes to be send
            let write_len = (free_size as usize).min(remaining_bytes.len());
            // let end_index= total_sent + write_len;
            let send_batch = &remaining_bytes[..write_len];

            // but on consequent send calls, we leave that to send
            let sent = self.send(bus, send_batch)?;

            total_sent += sent;

            // update the `free_size` of the TX buffer after we've sent some bytes
            free_size = self.socket.get_tx_free_size(bus)?;
        }

        Ok(())
    }

    /// Send buffer should be less than [`u16::MAX`].
    ///
    /// # Returns
    ///
    /// The amount of bytes that were sent. The caller should make sure to
    /// check and send the rest of the `send_buffer` data.
    fn send<SpiBus: Bus>(
        &self,
        bus: &mut SpiBus,
        send_buffer: &[u8],
    ) -> NbResult<usize, UdpSocketError<SpiBus::Error>> {
        let free_size = self.socket.get_tx_free_size(bus)?;

        // ensure write is currently possible
        if free_size == 0 {
            // TODO: better error for no more space in TX buffer
            return Err(NbError::WouldBlock);
        }

        // check the size of the data buffer and limit it accordingly to the available (free) TX buffer size.
        let write_data = if send_buffer.len() < free_size as usize {
            send_buffer
        } else {
            &send_buffer[..(free_size as usize)]
        };

        #[cfg(feature = "defmt")]
        defmt::debug!(
            "Prepare for sending {} bytes out of {} ({} free TX buffer size)",
            write_data.len(),
            send_buffer.len(),
            free_size,
        );
        // Append the data to the write buffer after the current write pointer.
        let write_pointer = self.socket.get_tx_write_pointer(bus)?;

        // #[cfg(feature = "defmt")]
        // defmt::debug!("TX read at {} TX Write at {}", self.socket.get_tx_read_pointer(bus)?, write_pointer);

        #[cfg(feature = "defmt")]
        defmt::debug!("TX Buffer at {}, write pointer: {}", self.socket.tx_buffer(), write_pointer);
        
        // Write data into the buffer and update the writer pointer.
        bus.write_frame(self.socket.tx_buffer(), write_pointer, write_data)?;
        // this will wrap the pointer accordingly to the TX `free_size`.
        // safe to cast to `u16` because the maximum buffer size in w5500 is 16 KB!
        self.socket
            .set_tx_write_pointer(bus, write_pointer.wrapping_add(write_data.len() as u16))?;

        #[cfg(feature = "defmt")]
        defmt::debug!("NEW TX Write at {}", write_pointer.wrapping_add(write_data.len() as u16));

        // Send the data.
        self.socket.command(bus, socketn::Command::Send)?;

        // #[cfg(feature = "defmt")]
        // defmt::debug!("NEW TX read at {} TX Write at {}", self.socket.get_tx_read_pointer(bus)?, write_pointer.wrapping_add(write_data.len() as u16));

        // wait for reaching out the TX write pointer (we've sent all the data in the buffer)
        // #[cfg(feature = "defmt")]
        // for i in 0..10 {
        //     let tx_read = self.socket.get_tx_read_pointer(bus)?;
        //     let tx_write = self.socket.get_tx_write_pointer(bus)?;

        //     defmt::debug!("TX Read at {} TX Write at {}", tx_read, tx_write);
        // }

        loop {
            let tx_read = self.socket.get_tx_read_pointer(bus)?;
            let tx_write = self.socket.get_tx_write_pointer(bus)?;
            if tx_read == tx_write {
                if self.socket.has_interrupt(bus, socketn::Interrupt::SendOk)? {
                    #[cfg(feature = "defmt")]
                    defmt::debug!("has interrupt for SendOk");

                    self.socket
                        .reset_interrupt(bus, socketn::Interrupt::SendOk)?;
                    return Ok(write_data.len());
                } else if self
                    .socket
                    .has_interrupt(bus, socketn::Interrupt::Timeout)?
                {
                    #[cfg(feature = "defmt")]
                    defmt::debug!("has interrupt for Timeout");

                    self.socket
                        .reset_interrupt(bus, socketn::Interrupt::Timeout)?;
                    return Err(NbError::Other(UdpSocketError::WriteTimeout));
                }
            }
        }

        Ok(write_data.len())
    }

    /// Sets a new destination before performing the send operation.
    ///
    /// # Returns
    /// The amount of bytes that were sent. The caller should make sure to
    /// check and send the rest of the `send_buffer` data.
    fn send_to<SpiBus: Bus>(
        &mut self,
        bus: &mut SpiBus,
        remote: SocketAddrV4,
        send_buffer: &[u8],
    ) -> NbResult<(), UdpSocketError<SpiBus::Error>> {
        self.set_destination(bus, remote)?;

        self.send_all(bus, send_buffer)
    }

    /// Receive data and mutate the `receive_buffer`.
    ///
    /// `receive_buffer` will only be used for receiving the packet data.
    /// The [`UdpHeader`]'s - [`SocketAddrV4`] and `len` will be read separately from the buffer.
    /// Note that the header is part of the internal RX buffer so `receive_buffer` can be smaller
    /// in size by 8 bytes.
    ///
    /// If the packet len is larger than the internal RX buffer, the data will be truncated.
    ///
    /// If [`Interrupt::Receive`] is not set, it will always return [`NbError::WouldBlock`].
    ///
    ///
    /// Packet frame, as described in W5200 docs section 5.2.2.1:
    ///
    /// ```text
    /// |<-- read_pointer                                 read_pointer + received_size -->|
    /// | Destination IP Address | Destination Port | Byte Size of DATA | Actual DATA ... |
    /// |    --- 4 Bytes ---     |  --- 2 Bytes --- |  --- 2 Bytes ---  |      ....       |
    /// ```
    fn receive<SpiBus: Bus>(
        &mut self,
        bus: &mut SpiBus,
        receive_buffer: &mut [u8],
    ) -> NbResult<(usize, UdpHeader), UdpSocketError<SpiBus::Error>> {
        if !self
            .socket
            .has_interrupt(bus, socketn::Interrupt::Receive)?
        {
            return Err(NbError::WouldBlock);
        }

        let rx_size = self.socket.get_receive_size(bus)? as usize;
        if rx_size == 0 {
            return Err(NbError::WouldBlock);
        }
        let buffer_size = receive_buffer.len();
        if buffer_size == 0 {
            return Err(NbError::Other(UdpSocketError::BufferOverflow));
        }

        // the amount of bytes we are able to read, the rest will be truncated by setting the RX read pointer
        // to the end of the RX buffer.
        let read_size = rx_size.min(buffer_size);

        let read_buffer = &mut receive_buffer[..read_size];

        // Read from the RX ring buffer.
        let read_pointer = self.socket.get_rx_read_pointer(bus)?;

        let mut header = [0u8; 8];
        // read enough data for the headers - remote SocketAddr & Packet size
        bus.read_frame(self.socket.rx_buffer(), read_pointer, &mut header)?;
        let origin_addr = SocketAddrV4::new(
            Ipv4Addr::new(header[0], header[1], header[2], header[3]),
            u16::from_be_bytes([header[4], header[5]]),
        );

        let packet_len: usize = u16::from_be_bytes([header[6], header[7]]).into();
        let udp_header = UdpHeader {
            origin: origin_addr,
            len: packet_len,
        };

        let data_read_pointer = read_pointer.wrapping_add(8);

        /// read the rest of the packet's data that can fit in the buffer
        bus.read_frame(self.socket.rx_buffer(), read_pointer, read_buffer)?;

        // Set the RX point after the `rx_size`, truncating any bytes that the
        // `receiving_buffer` was not able to fit
        // it's safe to cast `rx_size` to u16 as the maximum RX buffer is
        // 16 KB (`16384` maximum value) < u16::MAX
        self.socket
            .set_rx_read_pointer(bus, read_pointer.wrapping_add(rx_size as u16))?;

        // > RECV completes the processing of the received data in Socket n RX
        // > Buffer by using a RX read pointer register (Sn_RX_RD).
        self.socket.command(bus, socketn::Command::Receive)?;
        // TODO: is this command `Open` really necessary if the socket is already set as Open?
        self.socket.command(bus, socketn::Command::Open)?;

        // Reset the Receive interrupt
        self.socket
            .reset_interrupt(bus, socketn::Interrupt::Receive)?;

        #[cfg(feature = "defmt")]
        defmt::debug!(
            "Received {} bytes of maximum {} RX buffer size from packet with length {}",
            read_size,
            rx_size,
            packet_len
        );

        Ok((read_size, udp_header))
    }

    fn close<SpiBus: Bus>(&self, bus: &mut SpiBus) -> Result<(), UdpSocketError<SpiBus::Error>> {
        self.socket.set_mode(bus, socketn::Protocol::Closed)?;
        self.socket.command(bus, socketn::Command::Close)?;
        Ok(())
    }

    /// returns the index of the socket
    #[inline]
    pub fn index(&self) -> u8 {
        self.socket.index
    }
}

#[derive(Debug)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum UdpSocketError<E: Debug> {
    NoMoreSockets,
    UnsupportedAddress,
    /// Reading the entire packet will cause the buffer to overflow.
    ///
    /// Use a larger than the packet size buffer.
    BufferOverflow,
    Other(#[cfg_attr(feature = "defmt", defmt(Debug2Format))] E),
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
        socket.send_all(&mut self.bus, buffer)?;

        Ok(())
    }

    fn receive(
        &mut self,
        socket: &mut Self::UdpSocket,
        buffer: &mut [u8],
    ) -> nb::Result<(usize, SocketAddr), Self::Error> {
        let (received, udp_header) = socket.receive(&mut self.bus, buffer)?;

        Ok((received, SocketAddr::V4(udp_header.origin)))
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

#[cfg(test)]
mod test {
    #[test]
    fn send_all_with_data_overflowing_the_tx_buffer() {}
}
