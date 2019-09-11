pub const COMMON: u8 = 0;
pub mod common {
    pub const MODE: u16 = 0x0;
    pub const GATEWAY: u16 = 0x1;
    pub const SUBNET_MASK: u16 = 0x5;
    pub const MAC: u16 = 0x9;
    pub const IP: u16 = 0xF;
    pub const VERSION: u16 = 0x39;
}

pub const SOCKET0: u8 = 0b000_00001;
pub const SOCKET0_BUFFER_TX: u8 = 0b000_00010;
pub const SOCKET0_BUFFER_RX: u8 = 0b000_00011;

pub const SOCKET1: u8 = 0b000_00101;
pub const SOCKET1_BUFFER_TX: u8 = 0b000_00110;
pub const SOCKET1_BUFFER_RX: u8 = 0b000_00111;

pub const SOCKET2: u8 = 0b000_01001;
pub const SOCKET2_BUFFER_TX: u8 = 0b000_01010;
pub const SOCKET2_BUFFER_RX: u8 = 0b000_01011;

pub const SOCKET3: u8 = 0b000_01101;
pub const SOCKET3_BUFFER_TX: u8 = 0b000_01110;
pub const SOCKET3_BUFFER_RX: u8 = 0b000_01111;

pub const SOCKET4: u8 = 0b000_10001;
pub const SOCKET4_BUFFER_TX: u8 = 0b000_10010;
pub const SOCKET4_BUFFER_RX: u8 = 0b000_10011;

pub const SOCKET5: u8 = 0b000_10101;
pub const SOCKET5_BUFFER_TX: u8 = 0b000_10110;
pub const SOCKET5_BUFFER_RX: u8 = 0b000_10111;

pub const SOCKET6: u8 = 0b000_11001;
pub const SOCKET6_BUFFER_TX: u8 = 0b000_11010;
pub const SOCKET6_BUFFER_RX: u8 = 0b000_11011;

pub const SOCKET7: u8 = 0b000_11101;
pub const SOCKET7_BUFFER_TX: u8 = 0b000_11110;
pub const SOCKET7_BUFFER_RX: u8 = 0b000_11111;

pub mod socketn {
    pub const MODE: u16 = 0x00;
    #[repr(u8)]
    pub enum Protocol {
        Udp = 0b10u8,
    }
    pub const COMMAND: u16 = 0x01;
    #[repr(u8)]
    pub enum Command {
        Receive = 0x40,
    }

    pub const INTERRUPT: u16 = 0x02;
    #[repr(u8)]
    pub enum Interrupt {
        SendOk = 0b10000u8,
    }

    pub const SOURCE_PORT: u16 = 0x04;

    pub const INTERRUPT_MASK: u16 = 0x2C;
    pub mod interrupt_mask {
        pub const RECEIVE: u8 = 0b100;
    }

    pub const RECEIVED_SIZE: u16 = 0x26;

    pub const RX_DATA_READ_POINTER: u16 = 0x28;
    pub const RX_DATA_WRITE_POINTER: u16 = 0x2A;
}
