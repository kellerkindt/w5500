use crate::bus::Bus;
use crate::register::socketn::Command;
use crate::socket::Socket;

pub(crate) struct RxCursor<'a, SpiBus>
where
    SpiBus: Bus,
{
    sock: &'a mut Socket,
    bus: &'a mut SpiBus,
    ptr: u16,
    size: u16,
}

impl<'a, SpiBus> RxCursor<'a, SpiBus>
where
    SpiBus: Bus,
{
    pub fn new(sock: &'a mut Socket, bus: &'a mut SpiBus) -> Result<Self, SpiBus::Error> {
        let size = sock.get_receive_size(bus)?;
        let ptr = sock.get_rx_read_pointer(bus)?;
        Ok(Self {
            sock,
            bus,
            ptr,
            size,
        })
    }

    #[inline]
    pub fn available(&self) -> u16 {
        self.size
    }

    /// Read up to `buf.len()` bytes. The actual number of bytes read is bounded by `available()`.
    pub fn read(&mut self, buf: &mut [u8]) -> Result<u16, SpiBus::Error> {
        if buf.is_empty() {
            return Ok(0);
        }

        let count = self.available().min(buf.len() as u16);
        self.bus
            .read_frame(self.sock.rx_buffer(), self.ptr, &mut buf[..count as _])?;
        Ok(self.skip(count))
    }

    /// Read up to `max` bytes. The actual number of bytes read is bounded by buf.len() and available().
    pub fn read_upto(&mut self, buf: &mut [u8], max: u16) -> Result<u16, SpiBus::Error> {
        let bounded_buf = if buf.len() > max as usize {
            &mut buf[..max as _]
        } else {
            buf
        };
        self.read(bounded_buf)
    }

    /// Skip up to count bytes. The actual number of bytes skipped is bounded by available().
    pub fn skip(&mut self, count: u16) -> u16 {
        let bounded_count = self.available().min(count);
        self.ptr += bounded_count;
        self.size -= bounded_count;
        bounded_count
    }

    /// Return ownership of the portion of the RX buffer that has already been read back to the
    /// chip and issue the next receive command.
    pub fn commit(mut self) -> Result<(), SpiBus::Error> {
        self.sock.set_rx_read_pointer(self.bus, self.ptr)?;
        self.sock.command(self.bus, Command::Receive)?;
        Ok(())
    }
}

pub(crate) struct TxCursor<'a, SpiBus>
where
    SpiBus: Bus,
{
    sock: &'a mut Socket,
    bus: &'a mut SpiBus,
    ptr: u16,
    size: u16,
}

impl<'a, SpiBus> TxCursor<'a, SpiBus>
where
    SpiBus: Bus,
{
    pub fn new(sock: &'a mut Socket, bus: &'a mut SpiBus) -> Result<Self, SpiBus::Error> {
        let size = sock.get_tx_free_size(bus)?;
        let ptr = sock.get_tx_write_pointer(bus)?;
        Ok(Self {
            sock,
            bus,
            ptr,
            size,
        })
    }

    #[inline]
    pub fn available(&self) -> u16 {
        self.size
    }

    /// Write all bytes in buf to the current TX buffer position and update the cursor position
    /// and remaining size on success.
    pub fn write(&mut self, buf: &[u8]) -> Result<u16, SpiBus::Error> {
        if buf.is_empty() || buf.len() > self.available() as _ {
            return Ok(0);
        }

        let count = buf.len() as u16;
        self.bus
            .write_frame(self.sock.tx_buffer(), self.ptr, &buf[..count as _])?;
        self.ptr += count;
        self.size -= count;
        Ok(count)
    }

    /// Pass ownership of the portion of the TX buffer that has already been written back to the
    /// chip and issue the next send command.
    pub fn commit(mut self) -> Result<(), SpiBus::Error> {
        self.sock.set_tx_write_pointer(self.bus, self.ptr)?;
        self.sock.command(self.bus, Command::Send)?;
        Ok(())
    }
}
