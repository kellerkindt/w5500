use crate::inactive_w5500::InactiveW5500;
use crate::uninitialized_w5500::UninitializedW5500;
use bit_field::BitArray;
use bus::{ActiveBus, ActiveFourWire, ActiveThreeWire, FourWire, ThreeWire};
use embedded_hal::digital::v2::OutputPin;
use embedded_hal::spi::FullDuplex;
use network::Network;
use register;
use socket::Socket;

pub struct W5500<SpiBus: ActiveBus, NetworkImpl: Network> {
    pub bus: SpiBus,
    network: NetworkImpl,
    sockets: [u8; 1],
}

pub enum ResetError<E> {
    SocketsNotReleased,
    Other(E),
}

impl<E> From<E> for ResetError<E> {
    fn from(error: E) -> ResetError<E> {
        ResetError::Other(error)
    }
}

impl<SpiBus: ActiveBus, NetworkImpl: Network> W5500<SpiBus, NetworkImpl> {
    pub fn new(bus: SpiBus, network: NetworkImpl) -> Self {
        W5500 {
            bus,
            network,
            sockets: [0b11111111],
        }
    }

    pub fn reset(mut self) -> Result<UninitializedW5500<SpiBus>, ResetError<SpiBus::Error>> {
        if self.sockets != [0b11111111] {
            Err(ResetError::SocketsNotReleased)
        } else {
            self.clear_mode()?;
            Ok(UninitializedW5500::new(self.bus))
        }
    }

    fn clear_mode(&mut self) -> Result<(), SpiBus::Error> {
        // reset bit
        let mut mode = [0b10000000];
        self.bus
            .write_frame(register::COMMON, register::common::MODE, &mut mode)?;
        Ok(())
    }

    pub fn take_socket(&mut self) -> Option<Socket> {
        for index in 0..8 {
            if self.sockets.get_bit(index) {
                self.sockets.set_bit(index, false);
                return Some(Socket::new(index as u8));
            }
        }
        None
    }

    pub fn release_socket(&mut self, socket: Socket) -> () {
        self.sockets.set_bit(socket.index.into(), true);
    }

    pub fn release(self) -> (SpiBus, NetworkImpl) {
        (self.bus, self.network)
    }
}

impl<Spi: FullDuplex<u8>, ChipSelect: OutputPin, NetworkImpl: Network>
    W5500<ActiveFourWire<Spi, ChipSelect>, NetworkImpl>
{
    pub fn deactivate(self) -> (InactiveW5500<FourWire<ChipSelect>, NetworkImpl>, Spi) {
        let (bus, spi) = self.bus.deactivate();
        (InactiveW5500::new(bus, self.network), spi)
    }
}

impl<Spi: FullDuplex<u8>, NetworkImpl: Network> W5500<ActiveThreeWire<Spi>, NetworkImpl> {
    pub fn deactivate(self) -> (InactiveW5500<ThreeWire, NetworkImpl>, Spi) {
        let (bus, spi) = self.bus.deactivate();
        (InactiveW5500::new(bus, self.network), spi)
    }
}
