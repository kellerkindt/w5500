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
        let raw_socket = Socket::new(0);

        // Configure the chip in MACRAW mode with MAC filtering.
        let mode: u8 = (1 << 7) | (register::socketn::Protocol::MacRaw as u8);

        bus.write_frame(raw_socket.register(), register::socketn::MODE, &[mode])?;
        raw_socket.command(&mut bus, register::socketn::Command::Open)?;

        Ok(Self { bus, raw_socket })
    }

    /// Read an ethernet frame from the device.
    ///
    /// # Args
    /// * `frame` - The location to store the received frame
    ///
    /// # Returns
    /// The number of bytes read into the provided frame buffer.
    pub fn read_frame(&mut self, frame: &mut [u8]) -> Result<usize, SpiBus::Error> {
        if !self
            .raw_socket
            .has_interrupt(&mut self.bus, register::socketn::Interrupt::Receive)?
        {
            return Ok(0);
        }

        let rx_size = self.raw_socket.get_receive_size(&mut self.bus)? as usize;

        let read_buffer = if rx_size > frame.len() {
            frame
        } else {
            &mut frame[..rx_size]
        };

        // Read from the RX ring buffer.
        let read_pointer = self.raw_socket.get_rx_read_pointer(&mut self.bus)?;
        self.bus
            .read_frame(self.raw_socket.rx_buffer(), read_pointer, read_buffer)?;
        self.raw_socket.set_rx_read_pointer(
            &mut self.bus,
            read_pointer.wrapping_add(read_buffer.len() as u16),
        )?;

        // Register the reception as complete.
        self.raw_socket
            .command(&mut self.bus, register::socketn::Command::Receive)?;
        self.raw_socket
            .reset_interrupt(&mut self.bus, register::socketn::Interrupt::Receive)?;

        Ok(read_buffer.len())
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
