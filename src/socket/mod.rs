use byteorder::{BigEndian, ByteOrder};

use crate::bus::ActiveBus;
use crate::register;
use crate::register::socketn;

pub trait Socket {
    fn is_owned_by(&self, sockets: &OwnedSockets) -> bool;
    fn register(&self) -> u8;
    fn tx_buffer(&self) -> u8;
    fn rx_buffer(&self) -> u8;

    fn set_mode<SpiBus: ActiveBus>(
        &self,
        bus: &mut SpiBus,
        mode: socketn::Protocol,
    ) -> Result<(), SpiBus::Error> {
        let mut mode = [mode as u8];
        block!(bus.transfer_frame(self.register(), socketn::MODE, true, &mut mode))?;
        Ok(())
    }

    fn reset_interrupt<SpiBus: ActiveBus>(
        &self,
        bus: &mut SpiBus,
        code: socketn::Interrupt,
    ) -> Result<(), SpiBus::Error> {
        let mut data = [code as u8];
        block!(bus.transfer_frame(self.register(), socketn::INTERRUPT, true, &mut data))?;
        Ok(())
    }

    fn set_source_port<SpiBus: ActiveBus>(
        &self,
        bus: &mut SpiBus,
        port: u16,
    ) -> Result<(), SpiBus::Error> {
        let mut data = [0u8; 2];
        BigEndian::write_u16(&mut data[..], port);
        block!(bus.transfer_frame(self.register(), socketn::SOURCE_PORT, true, &mut data))?;
        Ok(())
    }

    fn has_received<SpiBus: ActiveBus>(
        &self,
        bus: &mut SpiBus,
    ) -> Result<bool, SpiBus::Error> {
        let mut data = [0u8];
        block!(bus.transfer_frame(self.register(), socketn::INTERRUPT_MASK, true, &mut data))?;
        Ok(data[0] & socketn::interrupt_mask::RECEIVE != 0)
    }

    fn get_rx_read_pointer<SpiBus: ActiveBus>(
        &self,
        bus: &mut SpiBus,
    ) -> Result<u16, SpiBus::Error> {
        let mut data = [0u8; 2];
        block!(bus.transfer_frame(self.register(), socketn::RX_DATA_READ_POINTER, true, &mut data))?;
        Ok(BigEndian::read_u16(&data))
    }

    fn set_rx_read_pointer<SpiBus: ActiveBus>(&self, bus: &mut SpiBus, pointer: u16) -> Result<(), SpiBus::Error> {
        let mut data = [0u8; 2];
        BigEndian::write_u16(&mut data, pointer);
        block!(bus.transfer_frame(self.register(), socketn::RX_DATA_READ_POINTER, true, &mut data))?;
        Ok(())
    }

    fn command<SpiBus: ActiveBus>(&self, bus: &mut SpiBus, command: socketn::Command) -> Result<(), SpiBus::Error> {
        let mut data = [0u8; 2];
        BigEndian::write_u16(&mut data, command as u16);
        block!(bus.transfer_frame(self.register(), socketn::COMMAND, true, &mut data))?;
        Ok(())
    }

    fn get_receive_size<SpiBus: ActiveBus>(&self, bus: &mut SpiBus) -> Result<u16, SpiBus::Error> {
        loop {
            // Section 4.2 of datasheet, Sn_TX_FSR address docs indicate that read must be repeated until two sequential reads are stable
            let mut sample_0 = [0u8; 2];
            block!(bus.transfer_frame(self.register(), socketn::RECEIVED_SIZE, false, &mut sample_0))?;
            let mut sample_1 = [0u8; 2];
            block!(bus.transfer_frame(self.register(), socketn::RECEIVED_SIZE, false, &mut sample_1))?;
            if sample_0 == sample_1 && sample_0[0] >= 8 {
                break Ok(BigEndian::read_u16(&sample_0));
            }
        }
    }
}

pub type OwnedSockets = (
    Socket0,
    Socket1,
    Socket2,
    Socket3,
    Socket4,
    Socket5,
    Socket6,
    Socket7,
);
pub type Sockets<'a> = (
    &'a mut Socket0,
    &'a mut Socket1,
    &'a mut Socket2,
    &'a mut Socket3,
    &'a mut Socket4,
    &'a mut Socket5,
    &'a mut Socket6,
    &'a mut Socket7,
);

pub struct Socket0 {}
impl Socket for Socket0 {
    fn is_owned_by(&self, sockets: &OwnedSockets) -> bool {
        self as *const _ == &sockets.0 as *const _
    }
    fn register(&self) -> u8 {
        register::SOCKET0
    }
    fn tx_buffer(&self) -> u8 {
        register::SOCKET0_BUFFER_TX
    }
    fn rx_buffer(&self) -> u8 {
        register::SOCKET0_BUFFER_RX
    }
}
pub struct Socket1 {}
impl Socket for Socket1 {
    fn is_owned_by(&self, sockets: &OwnedSockets) -> bool {
        self as *const _ == &sockets.1 as *const _
    }
    fn register(&self) -> u8 {
        register::SOCKET1
    }
    fn tx_buffer(&self) -> u8 {
        register::SOCKET1_BUFFER_TX
    }
    fn rx_buffer(&self) -> u8 {
        register::SOCKET1_BUFFER_RX
    }
}
pub struct Socket2 {}
impl Socket for Socket2 {
    fn is_owned_by(&self, sockets: &OwnedSockets) -> bool {
        self as *const _ == &sockets.2 as *const _
    }
    fn register(&self) -> u8 {
        register::SOCKET2
    }
    fn tx_buffer(&self) -> u8 {
        register::SOCKET2_BUFFER_TX
    }
    fn rx_buffer(&self) -> u8 {
        register::SOCKET2_BUFFER_RX
    }
}
pub struct Socket3 {}
impl Socket for Socket3 {
    fn is_owned_by(&self, sockets: &OwnedSockets) -> bool {
        self as *const _ == &sockets.3 as *const _
    }
    fn register(&self) -> u8 {
        register::SOCKET3
    }
    fn tx_buffer(&self) -> u8 {
        register::SOCKET3_BUFFER_TX
    }
    fn rx_buffer(&self) -> u8 {
        register::SOCKET3_BUFFER_RX
    }
}
pub struct Socket4 {}
impl Socket for Socket4 {
    fn is_owned_by(&self, sockets: &OwnedSockets) -> bool {
        self as *const _ == &sockets.4 as *const _
    }
    fn register(&self) -> u8 {
        register::SOCKET4
    }
    fn tx_buffer(&self) -> u8 {
        register::SOCKET4_BUFFER_TX
    }
    fn rx_buffer(&self) -> u8 {
        register::SOCKET4_BUFFER_RX
    }
}
pub struct Socket5 {}
impl Socket for Socket5 {
    fn is_owned_by(&self, sockets: &OwnedSockets) -> bool {
        self as *const _ == &sockets.5 as *const _
    }
    fn register(&self) -> u8 {
        register::SOCKET5
    }
    fn tx_buffer(&self) -> u8 {
        register::SOCKET5_BUFFER_TX
    }
    fn rx_buffer(&self) -> u8 {
        register::SOCKET5_BUFFER_RX
    }
}
pub struct Socket6 {}
impl Socket for Socket6 {
    fn is_owned_by(&self, sockets: &OwnedSockets) -> bool {
        self as *const _ == &sockets.6 as *const _
    }
    fn register(&self) -> u8 {
        register::SOCKET6
    }
    fn tx_buffer(&self) -> u8 {
        register::SOCKET6_BUFFER_TX
    }
    fn rx_buffer(&self) -> u8 {
        register::SOCKET6_BUFFER_RX
    }
}
pub struct Socket7 {}
impl Socket for Socket7 {
    fn is_owned_by(&self, sockets: &OwnedSockets) -> bool {
        self as *const _ == &sockets.7 as *const _
    }
    fn register(&self) -> u8 {
        register::SOCKET7
    }
    fn tx_buffer(&self) -> u8 {
        register::SOCKET7_BUFFER_TX
    }
    fn rx_buffer(&self) -> u8 {
        register::SOCKET7_BUFFER_RX
    }
}
