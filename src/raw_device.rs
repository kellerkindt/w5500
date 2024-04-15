use crate::{bus::Bus, register, socket::Socket, uninitialized_device::InitializeError};

/// The W5500 operating in MACRAW mode to send and receive ethernet frames.
#[derive(Debug)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
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

    /// Enable one or more interrupts
    ///
    /// # Args
    /// * `which` - The interrupts to enable; see `register::socketn::Interrupt`
    ///             For instance, pass `Interrupt::Receive` to get interrupts
    ///             on packet reception only.
    ///
    pub fn enable_interrupts(&mut self, which: u8) -> Result<(), SpiBus::Error> {
        self.raw_socket.set_interrupt_mask(&mut self.bus, which)?;
        self.bus.write_frame(
            register::COMMON,
            register::common::SOCKET_INTERRUPT_MASK,
            &[1],
        )?;
        Ok(())
    }

    /// Clear pending interrupts
    ///
    /// If using RTIC or similar, this should be called from the
    /// interrupt handler. If not (i.e., if there's concern that this
    /// use of the SPI bus will clobber someone else's use), then you
    /// can mask the interrupt *at microcontroller level* in the
    /// interrupt handler, then call this from thread mode before
    /// unmasking again.
    pub fn clear_interrupts(&mut self) -> Result<(), SpiBus::Error> {
        self.raw_socket
            .reset_interrupt(&mut self.bus, register::socketn::Interrupt::All)
    }

    /// Disable all interrupts
    ///
    pub fn disable_interrupts(&mut self) -> Result<(), SpiBus::Error> {
        self.bus.write_frame(
            register::COMMON,
            register::common::SOCKET_INTERRUPT_MASK,
            &[0],
        )?;
        self.raw_socket.set_interrupt_mask(&mut self.bus, 0xFF)?;
        Ok(())
    }

    /// Read an ethernet frame from the device.
    ///
    /// # Args
    /// * `frame` - The location to store the received frame
    ///
    /// # Returns
    /// The number of bytes read into the provided frame buffer.
    pub fn read_frame(&mut self, frame: &mut [u8]) -> Result<usize, SpiBus::Error> {
        let mut rx_cursor = crate::cursor::RxCursor::new(&self.raw_socket, &mut self.bus)?;

        // Check if there is anything to receive.
        if rx_cursor.available() == 0 {
            return Ok(0);
        }

        // The W5500 specifies the size of the received ethernet frame in the first two bytes.
        // Refer to https://forum.wiznet.io/t/topic/979/2 for more information.
        let expected_frame_size = {
            let mut frame_bytes = [0u8; 2];
            assert!(rx_cursor.read(&mut frame_bytes[..])? == 2);

            u16::from_be_bytes(frame_bytes).saturating_sub(2)
        };

        let received_frame_size = rx_cursor.read_upto(frame, expected_frame_size)?;
        if received_frame_size < expected_frame_size {
            rx_cursor.skip(expected_frame_size - received_frame_size);
        }

        rx_cursor.commit()?;
        Ok(received_frame_size as _)
    }

    /// Write an ethernet frame to the device.
    ///
    /// # Args
    /// * `frame` - The ethernet frame to transmit.
    ///
    /// # Returns
    /// The number of bytes successfully transmitted from the provided buffer.
    pub fn write_frame(&mut self, frame: &[u8]) -> Result<usize, SpiBus::Error> {
        // Reset the transmission complete flag, we'll wait on it later.
        self.raw_socket
            .reset_interrupt(&mut self.bus, register::socketn::Interrupt::SendOk)?;

        let mut tx_cursor = crate::cursor::TxCursor::new(&self.raw_socket, &mut self.bus)?;
        let count = tx_cursor.write(frame)?;
        tx_cursor.commit()?;

        // Wait for the socket transmission to complete.
        while !self
            .raw_socket
            .has_interrupt(&mut self.bus, register::socketn::Interrupt::SendOk)?
        {}

        Ok(count as _)
    }
}
