use crate::register;

pub trait Socket {
    fn is_owned_by(&self, sockets: &OwnedSockets) -> bool;
    fn register(&self) -> u8;
    fn tx_buffer(&self) -> u8;
    fn rx_buffer(&self) -> u8;
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
        register::socket0::TX_BUFFER
    }
    fn rx_buffer(&self) -> u8 {
        register::socket0::RX_BUFFER
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
        register::socket1::TX_BUFFER
    }
    fn rx_buffer(&self) -> u8 {
        register::socket1::RX_BUFFER
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
        register::socket2::TX_BUFFER
    }
    fn rx_buffer(&self) -> u8 {
        register::socket2::RX_BUFFER
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
        register::socket3::TX_BUFFER
    }
    fn rx_buffer(&self) -> u8 {
        register::socket3::RX_BUFFER
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
        register::socket4::TX_BUFFER
    }
    fn rx_buffer(&self) -> u8 {
        register::socket4::RX_BUFFER
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
        register::socket5::TX_BUFFER
    }
    fn rx_buffer(&self) -> u8 {
        register::socket5::RX_BUFFER
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
        register::socket6::TX_BUFFER
    }
    fn rx_buffer(&self) -> u8 {
        register::socket6::RX_BUFFER
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
        register::socket7::TX_BUFFER
    }
    fn rx_buffer(&self) -> u8 {
        register::socket7::RX_BUFFER
    }
}
