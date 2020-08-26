use crate::{Error, TcpSocket, W5500};
use embedded_hal::blocking::spi::{Transfer, Write};
use embedded_hal::digital::v2::OutputPin;

pub struct Interface<CS: OutputPin, SPI: Transfer<u8> + Write<u8>> {
    w5500: core::cell::RefCell<W5500<CS, SPI>>,
}

impl<CSE, SPIE, CS, SPI> Interface<CS, SPI>
where
    SPI: Transfer<u8, Error = SPIE> + Write<u8, Error = SPIE>,
    CS: OutputPin<Error = CSE>,
{
    pub fn new(w5500: W5500<CS, SPI>) -> Self {
        Self {
            w5500: core::cell::RefCell::new(w5500),
        }
    }
}

impl<CSE, SPIE, CS, SPI> embedded_nal::TcpStack for Interface<CS, SPI>
where
    SPI: Transfer<u8, Error = SPIE> + Write<u8, Error = SPIE>,
    CS: OutputPin<Error = CSE>,
    CSE: core::fmt::Debug,
    SPIE: core::fmt::Debug,
{
    type TcpSocket = TcpSocket;
    type Error = Error<SPIE, CSE>;

    fn open(&self, _mode: embedded_nal::Mode) -> Result<Self::TcpSocket, Self::Error> {
        let socket = self.w5500.borrow_mut().open_tcp()?;
        Ok(socket)
    }

    fn connect(
        &self,
        socket: Self::TcpSocket,
        remote: embedded_nal::SocketAddr,
    ) -> Result<Self::TcpSocket, Self::Error> {
        match remote {
            embedded_nal::SocketAddr::V4(remote) => {
                self.w5500
                    .borrow_mut()
                    .connect_tcp(socket, remote.ip().clone(), remote.port())
            }
            embedded_nal::SocketAddr::V6(_) => Err(Self::Error::Unsupported),
        }
    }

    fn is_connected(&self, socket: &Self::TcpSocket) -> Result<bool, Self::Error> {
        self.w5500.borrow_mut().is_connected(socket)
    }

    fn write(&self, socket: &mut Self::TcpSocket, buffer: &[u8]) -> nb::Result<usize, Self::Error> {
        let count = self.w5500.borrow_mut().send(socket, buffer)?;
        Ok(count)
    }

    fn read(
        &self,
        socket: &mut Self::TcpSocket,
        buffer: &mut [u8],
    ) -> nb::Result<usize, Self::Error> {
        let count = self.w5500.borrow_mut().recv(socket, buffer)?;
        Ok(count)
    }

    fn close(&self, socket: Self::TcpSocket) -> Result<(), Self::Error> {
        if self.is_connected(&socket)? {
            self.w5500.borrow_mut().disconnect(&socket)?;
        }

        self.w5500.borrow_mut().close(socket)?;

        Ok(())
    }
}
