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
pub mod socket0 {
	pub const TX_BUFFER: u8 = 0b000_00010;
	pub const RX_BUFFER: u8 = 0b000_00011;
}

pub const SOCKET1: u8 = 0b000_00101;
pub mod socket1 {
	pub const TX_BUFFER: u8 = 0b000_00110;
	pub const RX_BUFFER: u8 = 0b000_00111;
}

pub const SOCKET2: u8 = 0b000_01001;
pub mod socket2 {
	pub const TX_BUFFER: u8 = 0b000_01010;
	pub const RX_BUFFER: u8 = 0b000_01011;
}

pub const SOCKET3: u8 = 0b000_01101;
pub mod socket3 {
	pub const TX_BUFFER: u8 = 0b000_01110;
	pub const RX_BUFFER: u8 = 0b000_01111;
}

pub const SOCKET4: u8 = 0b000_10001;
pub mod socket4 {
	pub const TX_BUFFER: u8 = 0b000_10010;
	pub const RX_BUFFER: u8 = 0b000_10011;
}

pub const SOCKET5: u8 = 0b000_10101;
pub mod socket5 {
	pub const TX_BUFFER: u8 = 0b000_10110;
	pub const RX_BUFFER: u8 = 0b000_10111;
}

pub const SOCKET6: u8 = 0b000_11001;
pub mod socket6 {
	pub const TX_BUFFER: u8 = 0b000_11010;
	pub const RX_BUFFER: u8 = 0b000_11011;
}

pub const SOCKET7: u8 = 0b000_11101;
pub mod socket7 {
	pub const TX_BUFFER: u8 = 0b000_11110;
	pub const RX_BUFFER: u8 = 0b000_11111;
}