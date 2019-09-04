pub trait Socket {
    fn is_owned_by(&self, sockets: OwnedSockets) -> bool;
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
    fn is_owned_by(&self, sockets: OwnedSockets) -> bool {
        self as *const _ == &sockets.0 as *const _
    }
    fn register(&self) -> u8 {
        0b000_00001
    }
    fn tx_buffer(&self) -> u8 {
        0b000_00010
    }
    fn rx_buffer(&self) -> u8 {
        0b000_00011
    }
}
pub struct Socket1 {}
impl Socket for Socket1 {
    fn is_owned_by(&self, sockets: OwnedSockets) -> bool {
        self as *const _ == &sockets.1 as *const _
    }
    fn register(&self) -> u8 {
        0b000_00101
    }
    fn tx_buffer(&self) -> u8 {
        0b000_00110
    }
    fn rx_buffer(&self) -> u8 {
        0b000_00111
    }
}
pub struct Socket2 {}
impl Socket for Socket2 {
    fn is_owned_by(&self, sockets: OwnedSockets) -> bool {
        self as *const _ == &sockets.2 as *const _
    }
    fn register(&self) -> u8 {
        0b000_01001
    }
    fn tx_buffer(&self) -> u8 {
        0b000_01010
    }
    fn rx_buffer(&self) -> u8 {
        0b000_01011
    }
}
pub struct Socket3 {}
impl Socket for Socket3 {
    fn is_owned_by(&self, sockets: OwnedSockets) -> bool {
        self as *const _ == &sockets.3 as *const _
    }
    fn register(&self) -> u8 {
        0b000_01101
    }
    fn tx_buffer(&self) -> u8 {
        0b000_01110
    }
    fn rx_buffer(&self) -> u8 {
        0b000_01111
    }
}
pub struct Socket4 {}
impl Socket for Socket4 {
    fn is_owned_by(&self, sockets: OwnedSockets) -> bool {
        self as *const _ == &sockets.4 as *const _
    }
    fn register(&self) -> u8 {
        0b000_10001
    }
    fn tx_buffer(&self) -> u8 {
        0b000_10010
    }
    fn rx_buffer(&self) -> u8 {
        0b000_10011
    }
}
pub struct Socket5 {}
impl Socket for Socket5 {
    fn is_owned_by(&self, sockets: OwnedSockets) -> bool {
        self as *const _ == &sockets.5 as *const _
    }
    fn register(&self) -> u8 {
        0b000_10101
    }
    fn tx_buffer(&self) -> u8 {
        0b000_10110
    }
    fn rx_buffer(&self) -> u8 {
        0b000_10111
    }
}
pub struct Socket6 {}
impl Socket for Socket6 {
    fn is_owned_by(&self, sockets: OwnedSockets) -> bool {
        self as *const _ == &sockets.6 as *const _
    }
    fn register(&self) -> u8 {
        0b000_11001
    }
    fn tx_buffer(&self) -> u8 {
        0b000_11010
    }
    fn rx_buffer(&self) -> u8 {
        0b000_11011
    }
}
pub struct Socket7 {}
impl Socket for Socket7 {
    fn is_owned_by(&self, sockets: OwnedSockets) -> bool {
        self as *const _ == &sockets.7 as *const _
    }
    fn register(&self) -> u8 {
        0b000_11101
    }
    fn tx_buffer(&self) -> u8 {
        0b000_11110
    }
    fn rx_buffer(&self) -> u8 {
        0b000_11111
    }
}
