use crate::{bus::Bus, register, socket::Socket, uninitialized_device::InitializeError};

/// The W5500 operating in MACRAW mode to send and receive ethernet frames.
pub struct RawDevice<SpiBus: Bus> {
    bus: SpiBus,
    raw_socket: Socket,
}

impl<SpiBus: Bus> RawDevice<SpiBus> {
    /// Create the raw device.
    ///
    /// # Note
    /// The device is configured with MAC filtering enabled.
    ///
    /// # Args
    /// * `bus` - The bus to communicate with the device.
    pub(crate) fn new(mut bus: SpiBus) -> Result<Self, InitializeError<SpiBus::Error>> {
        // Set the raw socket to 16KB RX/TX buffer space.
        let raw_socket = Socket::new(0);
        bus.write_frame(raw_socket.register(), register::socketn::TXBUF_SIZE, &[16])?;
        bus.write_frame(raw_socket.register(), register::socketn::RXBUF_SIZE, &[16])?;

        // Set all socket buffers to 0KB size.
        for socket_index in 1..8 {
            let socket = Socket::new(socket_index);
            bus.write_frame(socket.register(), register::socketn::TXBUF_SIZE, &[0])?;
            bus.write_frame(socket.register(), register::socketn::RXBUF_SIZE, &[0])?;
        }

        // Configure the chip in MACRAW mode with MAC filtering.
        let mode: u8 = (1 << 7) | // MAC address filtering
                       (register::socketn::Protocol::MacRaw as u8);

        bus.write_frame(raw_socket.register(), register::socketn::MODE, &[mode])?;
        raw_socket.command(&mut bus, register::socketn::Command::Open)?;

        Ok(Self { bus, raw_socket })
    }

    // Read bytes from the RX buffer.
    //
    // # Args
    // * `buffer` - The location to read data into. The length of this slice determines how much
    // data is read.
    // * `offset` - The offset into current RX data to start reading from in bytes.
    //
    // # Returns
    // The number of bytes successfully read.
    fn read_bytes(&mut self, buffer: &mut [u8], offset: u16) -> Result<usize, SpiBus::Error> {
        let rx_size = self.raw_socket.get_receive_size(&mut self.bus)? as usize;

        let read_buffer = if rx_size > buffer.len() + offset as usize {
            buffer
        } else {
            &mut buffer[..rx_size - offset as usize]
        };

        let read_pointer = self
            .raw_socket
            .get_rx_read_pointer(&mut self.bus)?
            .wrapping_add(offset);
        self.bus
            .read_frame(self.raw_socket.rx_buffer(), read_pointer, read_buffer)?;
        self.raw_socket.set_rx_read_pointer(
            &mut self.bus,
            read_pointer.wrapping_add(read_buffer.len() as u16),
        )?;

        Ok(read_buffer.len())
    }

    /// Read an ethernet frame from the device.
    ///
    /// # Args
    /// * `frame` - The location to store the received frame
    ///
    /// # Returns
    /// The number of bytes read into the provided frame buffer.
    pub fn read_frame(&mut self, frame: &mut [u8]) -> Result<usize, SpiBus::Error> {
        // Check if there is anything to receive.
        let rx_size = self.raw_socket.get_receive_size(&mut self.bus)? as usize;
        if rx_size == 0 {
            return Ok(0);
        }

        // The W5500 specifies the size of the received ethernet frame in the first two bytes.
        // Refer to https://forum.wiznet.io/t/topic/979/2 for more information.
        let expected_frame_size: usize = {
            let mut frame_bytes = [0u8; 2];
            assert!(self.read_bytes(&mut frame_bytes[..], 0)? == 2);

            u16::from_be_bytes(frame_bytes) as usize - 2
        };

        // Read the ethernet frame
        let read_buffer = if frame.len() > expected_frame_size {
            &mut frame[..expected_frame_size]
        } else {
            frame
        };

        let received_frame_size = self.read_bytes(read_buffer, 2)?;

        // Register the reception as complete.
        self.raw_socket
            .command(&mut self.bus, register::socketn::Command::Receive)?;

        // If we couldn't read the whole frame, drop it instead.
        if received_frame_size < expected_frame_size {
            Ok(0)
        } else {
            Ok(received_frame_size)
        }
    }

    /// Write an ethernet frame to the device.
    ///
    /// # Args
    /// * `frame` - The ethernet frame to transmit.
    ///
    /// # Returns
    /// The number of bytes successfully transmitted from the provided buffer.
    pub fn write_frame(&mut self, frame: &[u8]) -> Result<usize, SpiBus::Error> {
        let max_size = self.raw_socket.get_tx_free_size(&mut self.bus)? as usize;

        let write_data = if frame.len() < max_size {
            frame
        } else {
            &frame[..max_size]
        };

        // Append the data to the write buffer after the current write pointer.
        let write_pointer = self.raw_socket.get_tx_write_pointer(&mut self.bus)?;

        // Write data into the buffer and update the writer pointer.
        self.bus
            .write_frame(self.raw_socket.tx_buffer(), write_pointer, write_data)?;
        self.raw_socket.set_tx_write_pointer(
            &mut self.bus,
            write_pointer.wrapping_add(write_data.len() as u16),
        )?;

        // Wait for the socket transmission to complete.
        self.raw_socket
            .reset_interrupt(&mut self.bus, register::socketn::Interrupt::SendOk)?;

        self.raw_socket
            .command(&mut self.bus, register::socketn::Command::Send)?;

        while !self
            .raw_socket
            .has_interrupt(&mut self.bus, register::socketn::Interrupt::SendOk)?
        {}

        Ok(write_data.len())
    }
}
