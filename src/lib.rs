#![no_std]
#![allow(unused)]
#![feature(const_fn)]

extern crate byteorder;
extern crate embedded_hal as hal;

#[macro_use(block)]
extern crate nb;

use hal::digital::OutputPin;
use hal::spi::FullDuplex;

use byteorder::ByteOrder;
use byteorder::BigEndian;


const COMMAND_READ  : u8 = 0x00 << 2;
const COMMAND_WRITE : u8 = 0x01 << 2;

const VARIABLE_DATA_LENGTH      : u8 = 0b_00;
const FIXED_DATA_LENGTH_1_BYTE  : u8 = 0b_01;
const FIXED_DATA_LENGTH_2_BYTES : u8 = 0b_10;
const FIXED_DATA_LENGTH_4_BYTES : u8 = 0b_11;

#[derive(Copy, Clone, PartialOrd, PartialEq, Default, Debug)]
pub struct IpAddress {
    address: [u8; 4]
}

impl IpAddress {
    pub fn new(a0: u8, a1: u8, a2: u8, a3: u8) -> IpAddress {
        IpAddress {
            address: [a0, a1, a2, a3]
        }
    }
}

impl ::core::fmt::Display for IpAddress {
    fn fmt(&self, f: &mut ::core::fmt::Formatter) -> ::core::fmt::Result {
        write!(f, "{}.{}.{}.{}",
               self.address[0],
               self.address[1],
               self.address[2],
               self.address[3],
        )
    }
}


#[derive(Copy, Clone, PartialOrd, PartialEq, Default, Debug)]
pub struct MacAddress {
    address: [u8; 6]
}

impl MacAddress {
    pub fn new(a0: u8, a1: u8, a2: u8, a3: u8, a4: u8, a5: u8) -> MacAddress {
        MacAddress {
            address: [a0, a1, a2, a3, a4, a5]
        }
    }
}

impl ::core::fmt::Display for MacAddress {
    fn fmt(&self, f: &mut ::core::fmt::Formatter) -> ::core::fmt::Result {
        write!(f, "{:02x}:{:02x}:{:02x}:{:02x}:{:02x}:{:02x}",
            self.address[0],
            self.address[1],
            self.address[2],
            self.address[3],
            self.address[4],
            self.address[5],
        )
    }
}


pub struct W5500<E, S: FullDuplex<u8, Error=E> + Sized, O: OutputPin>  {
    spi: S,
    cs:  O,
}

impl<E, S: FullDuplex<u8, Error=E> + Sized, O: OutputPin> W5500<E, S, O> {

    pub fn new(spi: S, cs: O) -> Result<W5500<E, S, O>, E> {
        W5500 {
            spi,
            cs,
        }.init()
    }

    fn init(mut self) -> Result<Self, E> {
        self.reset()?;
        self.set_mode(false, false, false, false)?;

        self.set_mac(&MacAddress::new(0x00, 0x08, 0xDC, 0x01, 0x02, 0x03))?;

        self.set_ip(&IpAddress::new(192, 168, 3, 222))?;
        self.set_subnet(&IpAddress::new(255, 255, 255, 0))?;
        self.set_gateway(&IpAddress::new(192, 168, 3, 1))?;

        Ok(self)
    }

    pub fn reset(&mut self) -> Result<(), E> {
        self.write_to(
            Register::CommonRegister(0x00_00_u16),
            &[
                0b1000_0000, // Mode Register (force reset)
            ]
        )
    }

    pub fn set_mode(&mut self, wol: bool, ping_block: bool, ppoe: bool, force_arp: bool) -> Result<(), E> {

        let mut mode = 0x00;

        if wol {
            mode |= (1 << 5);
        }

        if ping_block {
            mode |= (1 << 4);
        }

        if ppoe {
            mode |= (1 << 3);
        }

        if force_arp {
            mode |= (1 << 1);
        }

        self.write_to(
            Register::CommonRegister(0x00_00_u16),
            &[mode]
        )
    }

    pub fn set_gateway(&mut self, gateway: &IpAddress) -> Result<(), E> {
        self.write_to(
            Register::CommonRegister(0x00_01_u16),
            &gateway.address
        )
    }

    pub fn set_subnet(&mut self, subnet: &IpAddress) -> Result<(), E> {
        self.write_to(
            Register::CommonRegister(0x00_05_u16),
            &subnet.address
        )
    }

    pub fn set_mac(&mut self, mac: &MacAddress) -> Result<(), E> {
        self.write_to(
            Register::CommonRegister(0x00_09_u16),
            &mac.address
        )
    }

    pub fn get_mac(&mut self) -> Result<MacAddress, E> {
        let mut mac = MacAddress::default();
        self.read_from(
            Register::CommonRegister(0x00_09_u16),
            &mut mac.address
        )?;
        Ok(mac)
    }

    pub fn set_ip(&mut self, ip: &IpAddress) -> Result<(), E> {
        self.write_to(
            Register::CommonRegister(0x00_0F_u16),
            &ip.address
        )
    }

    pub fn send_udp(&mut self, socket: Socket, local_port: u16, host: &IpAddress, host_port: u16, data: &[u8]) -> Result<(), E> {
        // TODO not always socket 0
        // TODO check if in use


        self.write_to(
            socket.register_at(0x00_00),
            &[
                Protocol::UDP as u8, // Socket Mode Register
                SocketCommand::Open as u8 // Socket Command Regsiter
            ]
        );

        {
            let local_port = u16_to_be_bytes(local_port);
            let host_port = u16_to_be_bytes(host_port);

            self.write_to(
                socket.register_at(0x00_04),
                &[
                    local_port[0], local_port[1], // local port u16
                    0x00, 0x00, 0x00, 0x00, 0x00, 0x00, // destination mac
                    host.address[0], host.address[1], host.address[2], host.address[3], // target IP
                    host_port[0], host_port[1], // destination port (5354)
                ]
            )?;
        }


        let data_length = data.len() as u16;
        {
            let data_length = u16_to_be_bytes(data_length);

            self.write_to(
                socket.register_at(0x00_22),
                &[
                    0x00, 0x00,
                    data_length[0], data_length[1],
                ]
            );
        }


        self.write_to(
            socket.tx_register_at(0x00_00),
            &data[..data_length as usize]
        );

        self.write_to(
            socket.register_at(0x00_01),
            &[
                0x20 // SEND
            ]
        )
    }

    pub fn listen_udp(&mut self, socket: Socket, port: u16) -> Result<(), E> {
        self.write_u16(
            socket.register_at(0x00_04),
            port
        )?;
        self.write_to(
            socket.register_at(0x00_00),
            &[
                Protocol::UDP as u8, // Socket Mode Register
                SocketCommand::Open as u8 // Socket Command Regsiter
            ]
        )
    }

    /// TODO destination buffer has to be as large as the receive buffer or complete read is not guaranteed
    pub fn try_receive_udp(&mut self, socket: Socket, destination: &mut [u8]) -> Result<Option<(IpAddress, u16, usize)>, E> {
        if self.read_u8(socket.register_at(0x00_2c))? & 0x04 == 0 {
            return Ok(None);
        }
        let receive_size = loop {
            let s0 = self.read_u16(socket.register_at(0x00_26))?;
            let s1 = self.read_u16(socket.register_at(0x00_26))?;
            if s0 == s1 {
                break s0 as usize;
            }
        };
        if receive_size > 8 {
            let read_pointer = self.read_u16(socket.register_at(0x00_28))?;

            // |<-- read_pointer                                read_pointer + received_size -->|
            // |Destination IP Address | Destination Port | Byte Size of DATA | Actual DATA ... |
            // |   --- 4 Bytes ---     |  --- 2 Bytes --- |  --- 2 Bytes ---  |      ....       |

            let ip = self.read_ip(socket.rx_register_at(read_pointer))?;
            let port = self.read_u16(socket.rx_register_at(read_pointer+4))?;
            let data_length = destination.len().min(self.read_u16(socket.rx_register_at(read_pointer+6))? as usize);

            self.read_from(
                socket.rx_register_at(read_pointer+8),
                &mut destination[..data_length]
            )?;

            // self.read_from(socket.register_at(0x00_0C), &mut ip.address)?;
            // self.read_u16(socket.register_at(0x00_10))?;

            // reset
            self.write_u16(socket.register_at(0x00_28), read_pointer + receive_size as u16)?;
            self.write_u8(socket.register_at(0x00_01), SocketCommand::Recv as u8)?;

            Ok(Some((ip, port, data_length)))

        } else {
            Ok(None)
        }
    }

    pub fn read_u8(&mut self, register: Register) -> Result<u8, E> {
        let mut buffer = [0u8; 1];
        self.read_from(register, &mut buffer)?;
        Ok(buffer[0])
    }

    pub fn read_u16(&mut self, register: Register) -> Result<u16, E> {
        let mut buffer = [0u8; 2];
        self.read_from(register, &mut buffer)?;
        Ok(BigEndian::read_u16(&buffer))
    }

    pub fn read_ip(&mut self, register: Register) -> Result<IpAddress, E> {
        let mut ip = IpAddress::default();
        self.read_from(register, &mut ip.address)?;
        Ok(ip)
    }

    pub fn read_from(&mut self, register: Register, target: &mut [u8]) -> Result<(), E> {
        self.chip_select();
        let mut request = [0_u8, 0_u8, register.control_byte() | COMMAND_READ | VARIABLE_DATA_LENGTH];
        BigEndian::write_u16(&mut request[..2], register.address());
        let result = self
            .write_bytes(&request)
            .and_then(|_| self.read_bytes(target));
        self.chip_deselect();
        result
    }

    pub fn write_u8(&mut self, register: Register, value: u8) -> Result<(), E> {
        self.write_to(register, &[value])
    }

    pub fn write_u16(&mut self, register: Register, value: u16) -> Result<(), E> {
        let mut data = [0u8; 2];
        BigEndian::write_u16(&mut data, value);
        self.write_to(register, &data)
    }

    pub fn write_to(&mut self, register: Register, data: &[u8]) -> Result<(), E> {
        self.chip_select();
        let mut request = [0_u8, 0_u8, register.control_byte() | COMMAND_WRITE | VARIABLE_DATA_LENGTH];
        BigEndian::write_u16(&mut request[..2], register.address());
        let result = self
            .write_bytes(&request)
            .and_then(|_| self.write_bytes(data));
        self.chip_deselect();
        result
    }

    fn read_bytes(&mut self, bytes: &mut [u8]) -> Result<(), E> {
        for i in 0..bytes.len() {
            bytes[i] = self.read()?;
        }
        Ok(())
    }

    fn read(&mut self) -> Result<u8, E> {
        block!(self.spi.send(0x00))?;
        let result = block!(self.spi.read());
        result
    }

    fn write_bytes(&mut self, bytes: &[u8]) -> Result<(), E> {
        for b in bytes {
            self.write(*b)?;
        }
        Ok(())
    }

    fn write(&mut self, byte: u8) -> Result<(), E> {
        block!(self.spi.send(byte))?;
        block!(self.spi.read())?;
        Ok(())
    }

    fn chip_select(&mut self) {
        self.cs.set_low()
    }

    fn chip_deselect(&mut self) {
        self.cs.set_high()
    }

}

fn u16_to_be_bytes(u16: u16) -> [u8; 2] {
    let mut bytes = [0u8; 2];
    BigEndian::write_u16(&mut bytes, u16);
    bytes
}

#[repr(u8)]
#[derive(Copy, Clone, PartialEq, Debug)]
pub enum Protocol {
    TCP = 0b0001,
    UDP = 0b0010,
    MACRAW = 0b0100,
}

#[repr(u8)]
#[derive(Copy, Clone, PartialEq, Debug)]
pub enum SocketCommand {
    Open = 0x01,
    Listen = 0x02,
    Connect = 0x04,
    Disconnect = 0x08,
    Close = 0x10,
    Send = 0x20,
    SendMac = 0x21,
    SendKeep = 0x22,
    Recv = 0x40,
}

#[derive(Copy, Clone, PartialEq, PartialOrd, Debug)]
pub enum Socket {
    Socket0,
    Socket1,
    Socket2,
    Socket3,
    Socket4,
    Socket5,
    Socket6,
    Socket7,
}

impl Socket {
    fn tx_register_at(&self, address: u16) -> Register {
        match *self {
            Socket::Socket0 => Register::Socket0TxBuffer(address),
            Socket::Socket1 => Register::Socket1TxBuffer(address),
            Socket::Socket2 => Register::Socket2TxBuffer(address),
            Socket::Socket3 => Register::Socket3TxBuffer(address),
            Socket::Socket4 => Register::Socket4TxBuffer(address),
            Socket::Socket5 => Register::Socket5TxBuffer(address),
            Socket::Socket6 => Register::Socket6TxBuffer(address),
            Socket::Socket7 => Register::Socket7TxBuffer(address),
        }
    }

    fn rx_register_at(&self, address: u16) -> Register {
        match *self {
            Socket::Socket0 => Register::Socket0RxBuffer(address),
            Socket::Socket1 => Register::Socket1RxBuffer(address),
            Socket::Socket2 => Register::Socket2RxBuffer(address),
            Socket::Socket3 => Register::Socket3RxBuffer(address),
            Socket::Socket4 => Register::Socket4RxBuffer(address),
            Socket::Socket5 => Register::Socket5RxBuffer(address),
            Socket::Socket6 => Register::Socket6RxBuffer(address),
            Socket::Socket7 => Register::Socket7RxBuffer(address),
        }
    }

    fn register_at(&self, address: u16) -> Register {
        match *self {
            Socket::Socket0 => Register::Socket0Register(address),
            Socket::Socket1 => Register::Socket1Register(address),
            Socket::Socket2 => Register::Socket2Register(address),
            Socket::Socket3 => Register::Socket3Register(address),
            Socket::Socket4 => Register::Socket4Register(address),
            Socket::Socket5 => Register::Socket5Register(address),
            Socket::Socket6 => Register::Socket6Register(address),
            Socket::Socket7 => Register::Socket7Register(address),
        }
    }
}

#[derive(Copy, Clone, PartialEq, Debug)]
pub enum Register {
    CommonRegister(u16),

    Socket0Register(u16),
    Socket0TxBuffer(u16),
    Socket0RxBuffer(u16),

    Socket1Register(u16),
    Socket1TxBuffer(u16),
    Socket1RxBuffer(u16),

    Socket2Register(u16),
    Socket2TxBuffer(u16),
    Socket2RxBuffer(u16),

    Socket3Register(u16),
    Socket3TxBuffer(u16),
    Socket3RxBuffer(u16),

    Socket4Register(u16),
    Socket4TxBuffer(u16),
    Socket4RxBuffer(u16),

    Socket5Register(u16),
    Socket5TxBuffer(u16),
    Socket5RxBuffer(u16),

    Socket6Register(u16),
    Socket6TxBuffer(u16),
    Socket6RxBuffer(u16),

    Socket7Register(u16),
    Socket7TxBuffer(u16),
    Socket7RxBuffer(u16),
}

impl Register {
    fn control_byte(&self) -> u8 {
        match *self {
            Register::CommonRegister(_)  => 0b00000_000,

            Register::Socket0Register(_) => 0b00001_000,
            Register::Socket0TxBuffer(_) => 0b00010_000,
            Register::Socket0RxBuffer(_) => 0b00011_000,

            Register::Socket1Register(_) => 0b00101_000,
            Register::Socket1TxBuffer(_) => 0b00110_000,
            Register::Socket1RxBuffer(_) => 0b00111_000,

            Register::Socket2Register(_) => 0b01001_000,
            Register::Socket2TxBuffer(_) => 0b01010_000,
            Register::Socket2RxBuffer(_) => 0b01011_000,

            Register::Socket3Register(_) => 0b01101_000,
            Register::Socket3TxBuffer(_) => 0b01110_000,
            Register::Socket3RxBuffer(_) => 0b01111_000,

            Register::Socket4Register(_) => 0b10001_000,
            Register::Socket4TxBuffer(_) => 0b10010_000,
            Register::Socket4RxBuffer(_) => 0b10011_000,

            Register::Socket5Register(_) => 0b10101_000,
            Register::Socket5TxBuffer(_) => 0b10110_000,
            Register::Socket5RxBuffer(_) => 0b10111_000,

            Register::Socket6Register(_) => 0b11001_000,
            Register::Socket6TxBuffer(_) => 0b11010_000,
            Register::Socket6RxBuffer(_) => 0b11011_000,

            Register::Socket7Register(_) => 0b11101_000,
            Register::Socket7TxBuffer(_) => 0b11110_000,
            Register::Socket7RxBuffer(_) => 0b11111_000,
        }
    }

    fn address(&self) -> u16 {
        match *self {
            Register::CommonRegister(address) => address,

            Register::Socket0Register(address) => address,
            Register::Socket0TxBuffer(address) => address,
            Register::Socket0RxBuffer(address) => address,

            Register::Socket1Register(address) => address,
            Register::Socket1TxBuffer(address) => address,
            Register::Socket1RxBuffer(address) => address,

            Register::Socket2Register(address) => address,
            Register::Socket2TxBuffer(address) => address,
            Register::Socket2RxBuffer(address) => address,

            Register::Socket3Register(address) => address,
            Register::Socket3TxBuffer(address) => address,
            Register::Socket3RxBuffer(address) => address,

            Register::Socket4Register(address) => address,
            Register::Socket4TxBuffer(address) => address,
            Register::Socket4RxBuffer(address) => address,

            Register::Socket5Register(address) => address,
            Register::Socket5TxBuffer(address) => address,
            Register::Socket5RxBuffer(address) => address,

            Register::Socket6Register(address) => address,
            Register::Socket6TxBuffer(address) => address,
            Register::Socket6RxBuffer(address) => address,

            Register::Socket7Register(address) => address,
            Register::Socket7TxBuffer(address) => address,
            Register::Socket7RxBuffer(address) => address,
        }
    }
}
