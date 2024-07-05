use core::{convert::TryFrom, fmt::Debug};

use embedded_nal::{nb, IpAddr, Ipv4Addr, SocketAddr, SocketAddrV4, UdpClientStack, UdpFullStack};

use crate::{
    bus::Bus,
    device::{Device, State},
    register::socketn::{self, Status},
    socket::Socket,
};

/// W5500 UDP Header
///
/// Contains the destination IP address, port and the length of the data.
///
/// Packet frame, as described in W5200 docs section 5.2.2.1:
///
/// ```text
/// |<-- read_pointer                                 read_pointer + received_size -->|
/// | Destination IP Address | Destination Port | Byte Size of DATA | Actual DATA ... |
/// |    --- 4 Bytes ---     |  --- 2 Bytes --- |  --- 2 Bytes ---  |      ....       |
/// ```
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub struct UdpHeader {
    /// The origin socket address (IP address and port).
    #[cfg_attr(feature = "defmt", defmt(Debug2Format))]
    pub origin: SocketAddrV4,
    /// Length of the UDP packet in bytes.
    ///
    /// This may not be equal to the length of the data in the socket buffer if the UDP packet was truncated
    /// due to small buffer or the size of the internal W5500 RX buffer for the socket.
    pub len: usize,
}

impl UdpHeader {
    pub fn from_array(array: [u8; 8]) -> Self {
        Self::from(array)
    }
}

impl From<[u8; 8]> for UdpHeader {
    fn from(header: [u8; 8]) -> Self {
        let origin_addr = SocketAddrV4::new(
            Ipv4Addr::new(header[0], header[1], header[2], header[3]),
            u16::from_be_bytes([header[4], header[5]]),
        );

        let packet_len: usize = u16::from_be_bytes([header[6], header[7]]).into();

        Self {
            origin: origin_addr,
            len: packet_len,
        }
    }
}

#[derive(Debug)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub struct UdpSocket {
    socket: Socket,
    /// Whether or not there has been a destination set for the socket.
    #[cfg_attr(feature = "defmt", defmt(Debug2Format))]
    destination: Option<SocketAddrV4>,
    /// The local port of the socket
    port: u16,
}

impl UdpSocket {
    fn new(socket: Socket) -> Self {
        let socket_index = socket.index;
        UdpSocket {
            socket,
            destination: None,
            // TODO dynamically select a random port
            // chosen by fair dice roll.
            // guaranteed to be random.
            port: 49849 + u16::from(socket_index),
        }
    }

    fn open<SpiBus: Bus>(&self, bus: &mut SpiBus) -> Result<(), SpiBus::Error> {
        self.socket.command(bus, socketn::Command::Close)?;
        self.socket.reset_interrupt(bus, socketn::Interrupt::All)?;
        self.socket.set_source_port(bus, self.port)?;
        self.socket.set_mode(bus, socketn::Protocol::Udp)?;
        self.socket.set_interrupt_mask(
            bus,
            socketn::Interrupt::SendOk as u8 & socketn::Interrupt::Timeout as u8,
        )?;
        self.socket.command(bus, socketn::Command::Open)?;

        Ok(())
    }

    /// return the last set local port of the socket
    pub fn get_port(&self) -> u16 {
        self.port
    }

    pub fn set_port<SpiBus: Bus>(
        &mut self,
        bus: &mut SpiBus,
        local_port: u16,
    ) -> Result<(), UdpSocketError<SpiBus::Error>> {
        self.port = local_port;
        self.socket.set_source_port(bus, self.port)?;

        Ok(())
    }

    fn set_destination<SpiBus: Bus>(
        &mut self,
        bus: &mut SpiBus,
        remote: SocketAddrV4,
    ) -> Result<(), UdpSocketError<SpiBus::Error>> {
        // We set this variable to true when:
        /// - We have the same Socket address already set
        /// - We don't have any previous destination set
        ///
        // either no previous destination is set or a new one
        if self
            .destination
            .as_ref()
            .map(|dest| dest != &remote)
            .unwrap_or(true)
        {
            self.socket.set_destination_ip(bus, *remote.ip())?;
            self.socket.set_destination_port(bus, remote.port())?;

            // set the new destination after we've successfully set them on chip
            self.destination = Some(remote);
        }

        Ok(())
    }

    fn socket_send_all<SpiBus: Bus>(
        &self,
        bus: &mut SpiBus,
        send_buffer: &[u8],
    ) -> NbResult<(), UdpSocketError<SpiBus::Error>> {
        match Status::try_from(self.socket.get_status(bus)?) {
            Ok(Status::Udp) => {}
            Ok(status) => return Err(NbError::Other(UdpSocketError::SocketNotOpen)),
            Err(err) => return Err(NbError::Other(UdpSocketError::UnrecognisedStatus)),
        }

        if self.destination.is_none() {
            return Err(NbError::Other(UdpSocketError::DestinationNotSet));
        }

        let mut free_size = self.socket.get_tx_free_size(bus)?;

        // Ensure write is currently possible.
        // This should never be `0`
        if free_size == 0 {
            // If this happens, then `Send` was not called in previous operations.
            // FIXME: add a way to either:
            // - flush the buffer by clearing it up
            // - `Send` the data to its the destination - this might not be possible if the destination has changed.
            return Err(NbError::Other(UdpSocketError::BufferFull));
        }

        // on the first send, we rely on the free TX size and the length of the send_buffer
        let mut total_sent = 0;

        // if we've sent all bytes, exit the loop
        while total_sent != send_buffer.len() {
            let remaining_bytes = &send_buffer[total_sent..];

            // take the smaller value out of the free buffer length and
            // the length of the remaining bytes to be send
            let write_len = (free_size as usize).min(remaining_bytes.len());
            let send_batch = &remaining_bytes[..write_len];

            // but on consequent send calls, we leave that to send
            let sent = self.socket_send(bus, send_batch)?;

            total_sent += sent;

            // update the `free_size` of the TX buffer after we've sent some bytes
            free_size = self.socket.get_tx_free_size(bus)?;
        }

        Ok(())
    }

    /// Send buffer should be less than [`u16::MAX`].
    ///
    /// Will block the device until all data is sent.
    ///
    /// # Returns
    ///
    /// The amount of bytes that were sent. The caller should make sure to
    /// check and send the rest of the `send_buffer` data.
    fn socket_send<SpiBus: Bus>(
        &self,
        bus: &mut SpiBus,
        send_buffer: &[u8],
    ) -> NbResult<usize, UdpSocketError<SpiBus::Error>> {
        match Status::try_from(self.socket.get_status(bus)?) {
            Ok(Status::Udp) => {}
            Ok(status) => return Err(NbError::Other(UdpSocketError::SocketNotOpen)),
            Err(err) => return Err(NbError::Other(UdpSocketError::UnrecognisedStatus)),
        }

        // We need to have a set destination before sending data with this method.
        if self.destination.is_none() {
            return Err(NbError::Other(UdpSocketError::DestinationNotSet));
        }

        let free_size = self.socket.get_tx_free_size(bus)?;

        // Ensure write is currently possible.
        // This should never be `0`
        if free_size == 0 {
            // If this happens, then `Send` was not called in previous operations.
            // FIXME: add a way to either:
            // - flush the buffer by clearing it up
            // - `Send` the data to its the destination - this might not be possible if the destination has changed.
            return Err(NbError::Other(UdpSocketError::BufferFull));
        }

        // check the size of the data buffer and limit it accordingly to the available (free) TX buffer size.
        let write_data = if send_buffer.len() < free_size as usize {
            send_buffer
        } else {
            &send_buffer[..(free_size as usize)]
        };

        // Append the data to the write buffer after the current write pointer.
        let write_pointer = self.socket.get_tx_write_pointer(bus)?;

        // Write data into the buffer and update the writer pointer.
        bus.write_frame(self.socket.tx_buffer(), write_pointer, write_data)?;
        // this will wrap the pointer accordingly to the TX `free_size`.
        // safe to cast to `u16` because the maximum buffer size in w5500 is 16 KB!
        let new_write_pointer = write_pointer.wrapping_add(write_data.len() as u16);
        self.socket.set_tx_write_pointer(bus, new_write_pointer)?;

        self.block_send_command(bus)?;

        Ok(write_data.len())
    }

    /// Sets the socket to [`socketnCommand::Send`] and block flushes the TX buffer
    fn block_send_command<SpiBus: Bus>(
        &self,
        bus: &mut SpiBus,
    ) -> NbResult<(), UdpSocketError<SpiBus::Error>> {
        // Send the data.
        self.socket.command(bus, socketn::Command::Send)?;

        loop {
            match self.try_flush_tx(bus) {
                Err(NbError::WouldBlock) => {}
                result => return result,
            }
        }
    }

    fn try_flush_tx<SpiBus: Bus>(
        &self,
        bus: &mut SpiBus,
    ) -> NbResult<(), UdpSocketError<SpiBus::Error>> {
        let tx_read = self.socket.get_tx_read_pointer(bus)?;
        let tx_write = self.socket.get_tx_write_pointer(bus)?;

        if tx_read == tx_write {
            if self.socket.has_interrupt(bus, socketn::Interrupt::SendOk)? {
                self.socket
                    .reset_interrupt(bus, socketn::Interrupt::SendOk)?;

                return Ok(());
            }

            if self
                .socket
                .has_interrupt(bus, socketn::Interrupt::Timeout)?
            {
                self.socket
                    .reset_interrupt(bus, socketn::Interrupt::Timeout)?;

                return Err(NbError::Other(UdpSocketError::WriteTimeout));
            }
            // other interrupts are unreachable in UDP mode.

            return Ok(());
        }

        Err(NbError::WouldBlock)
    }

    /// Sets a new destination before performing the send operation.
    ///
    /// # Returns
    ///
    /// The amount of bytes that were sent. The caller should make sure to
    /// check and send the rest of the `send_buffer` data.
    fn socket_send_to<SpiBus: Bus>(
        &mut self,
        bus: &mut SpiBus,
        remote: SocketAddrV4,
        send_buffer: &[u8],
    ) -> NbResult<(), UdpSocketError<SpiBus::Error>> {
        self.set_destination(bus, remote)?;

        self.socket_send_all(bus, send_buffer)
    }

    /// Receive data and mutate the `receive_buffer`.
    ///
    /// `receive_buffer` will only be used for receiving the packet data.
    /// The [`UdpHeader`]'s - [`SocketAddrV4`] and `len` will be read separately from the buffer.
    /// Note that the header is part of the internal RX buffer so `receive_buffer` can be smaller
    /// in size by 8 bytes.
    ///
    /// If the packet len is larger than the provided RX buffer, the data will be truncated.
    ///
    /// If [`Interrupt::Receive`] is not set, it will always return [`NbError::WouldBlock`].
    fn socket_receive<SpiBus: Bus>(
        &mut self,
        bus: &mut SpiBus,
        receive_buffer: &mut [u8],
    ) -> NbResult<(usize, UdpHeader), UdpSocketError<SpiBus::Error>> {
        match Status::try_from(self.socket.get_status(bus)?) {
            Ok(Status::Udp) => {}
            Ok(status) => return Err(NbError::Other(UdpSocketError::SocketNotOpen)),
            Err(err) => return Err(NbError::Other(UdpSocketError::UnrecognisedStatus)),
        }

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
        // to the end (even if wrapped) of the RX buffer.
        let read_max_size = rx_size.min(buffer_size);

        // Read from the RX ring buffer.
        let read_pointer = self.socket.get_rx_read_pointer(bus)?;

        let mut header = [0u8; 8];
        // read enough data for the headers - remote SocketAddr & Packet size
        bus.read_frame(self.socket.rx_buffer(), read_pointer, &mut header)?;

        let udp_header = UdpHeader::from_array(header);

        // we have to exclude the header's bytes when reading the data we put in the buffer.
        let data_read_pointer = read_pointer.wrapping_add(8);

        // read to either the max RX Buffer length or passed buffer length.
        // just read to the end the packet
        let read_length = read_max_size.max(udp_header.len);

        /// the maximum amount of bytes we can read based on the smallest value of either:
        /// - the RX size of the socket
        /// - Buffer size
        let read_buffer = &mut receive_buffer[..read_length];

        // read the rest of the packet's data that can fit in the buffer
        bus.read_frame(self.socket.rx_buffer(), data_read_pointer, read_buffer)?;

        // Set the RX point after the `rx_size`, truncating any bytes that the
        // `receiving_buffer` was not able to fit
        // it's safe to cast `rx_size` to u16 as the maximum RX buffer is
        // 16 KB (`16384` maximum value) < u16::MAX
        self.socket
            .set_rx_read_pointer(bus, read_pointer.wrapping_add(rx_size as u16))?;

        // > RECV completes the processing of the received data in Socket n RX
        // > Buffer by using a RX read pointer register (Sn_RX_RD).
        self.socket.command(bus, socketn::Command::Receive)?;

        // Reset the Receive interrupt
        self.socket
            .reset_interrupt(bus, socketn::Interrupt::Receive)?;

        Ok((read_length, udp_header))
    }

    fn socket_close<SpiBus: Bus>(
        &self,
        bus: &mut SpiBus,
    ) -> Result<(), UdpSocketError<SpiBus::Error>> {
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

#[derive(Debug, Clone)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum UdpSocketError<E: Debug> {
    NoMoreSockets,
    /// Only IP V4 is supported
    UnsupportedAddress,
    /// Returned [`Status`] for the socket was not recognised.
    UnrecognisedStatus,
    /// Reading the entire packet will cause the buffer to overflow.
    ///
    /// Use a larger than the packet size buffer.
    BufferOverflow,
    /// The TX or RX buffer has no more space left.
    ///
    /// You either need to read in case of RX or send the `Send` command in case of TX.
    BufferFull,
    /// Destination for the socket hasn't been set
    ///
    /// Before using [`Device::send`] you first need to set the destination
    /// by connecting to a remote address.
    DestinationNotSet,
    /// Before sending any data over Udp you must first `bind` it to local port
    /// or `connect` to a remote address.
    SocketNotOpen,
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

impl<SpiBus, StateImpl> UdpClientStack for Device<SpiBus, StateImpl>
where
    SpiBus: Bus,
    StateImpl: State,
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
        let SocketAddr::V4(remote) = remote else {
            return Err(Self::Error::UnsupportedAddress);
        };
        socket.open(&mut self.bus)?;
        socket.set_destination(&mut self.bus, remote)?;
        Ok(())
    }

    fn send(&mut self, socket: &mut Self::UdpSocket, buffer: &[u8]) -> nb::Result<(), Self::Error> {
        socket.socket_send_all(&mut self.bus, buffer)?;
        Ok(())
    }

    fn receive(
        &mut self,
        socket: &mut Self::UdpSocket,
        buffer: &mut [u8],
    ) -> nb::Result<(usize, SocketAddr), Self::Error> {
        let (received, udp_header) = socket.socket_receive(&mut self.bus, buffer)?;

        Ok((received, SocketAddr::V4(udp_header.origin)))
    }

    fn close(&mut self, socket: Self::UdpSocket) -> Result<(), Self::Error> {
        socket.socket_close(&mut self.bus)?;
        self.release_socket(socket.socket);
        Ok(())
    }
}

impl<SpiBus, StateImpl> UdpFullStack for Device<SpiBus, StateImpl>
where
    SpiBus: Bus,
    StateImpl: State,
{
    fn bind(&mut self, socket: &mut Self::UdpSocket, local_port: u16) -> Result<(), Self::Error> {
        socket.set_port(&mut self.bus, local_port)?;
        socket.open(&mut self.bus)?;
        Ok(())
    }

    fn send_to(
        &mut self,
        socket: &mut Self::UdpSocket,
        remote: SocketAddr,
        buffer: &[u8],
    ) -> nb::Result<(), Self::Error> {
        let SocketAddr::V4(remote) = remote else {
            return Err(nb::Error::Other(Self::Error::UnsupportedAddress));
        };

        socket.socket_send_to(&mut self.bus, remote, buffer)?;
        Ok(())
    }
}
