#![no_std]
#![allow(unused)]

extern crate byteorder;
extern crate embedded_hal as hal;

#[macro_use(block)]
extern crate nb;

use hal::digital::OutputPin;
use hal::spi::FullDuplex;

use byteorder::BigEndian;
use byteorder::ByteOrder;

const COMMAND_READ: u8 = 0x00 << 2;
const COMMAND_WRITE: u8 = 0x01 << 2;

const VARIABLE_DATA_LENGTH: u8 = 0b_00;
const FIXED_DATA_LENGTH_1_BYTE: u8 = 0b_01;
const FIXED_DATA_LENGTH_2_BYTES: u8 = 0b_10;
const FIXED_DATA_LENGTH_4_BYTES: u8 = 0b_11;

#[derive(Copy, Clone, PartialOrd, PartialEq, Default, Debug)]
pub struct IpAddress {
    pub address: [u8; 4],
}

impl IpAddress {
    pub fn new(a0: u8, a1: u8, a2: u8, a3: u8) -> IpAddress {
        IpAddress {
            address: [a0, a1, a2, a3],
        }
    }
}

impl ::core::fmt::Display for IpAddress {
    fn fmt(&self, f: &mut ::core::fmt::Formatter) -> ::core::fmt::Result {
        write!(
            f,
            "{}.{}.{}.{}",
            self.address[0], self.address[1], self.address[2], self.address[3],
        )
    }
}

#[derive(Copy, Clone, PartialOrd, PartialEq, Default, Debug)]
pub struct MacAddress {
    pub address: [u8; 6],
}

impl MacAddress {
    pub fn new(a0: u8, a1: u8, a2: u8, a3: u8, a4: u8, a5: u8) -> MacAddress {
        MacAddress {
            address: [a0, a1, a2, a3, a4, a5],
        }
    }
}

impl ::core::fmt::Display for MacAddress {
    fn fmt(&self, f: &mut ::core::fmt::Formatter) -> ::core::fmt::Result {
        write!(
            f,
            "{:02x}:{:02x}:{:02x}:{:02x}:{:02x}:{:02x}",
            self.address[0],
            self.address[1],
            self.address[2],
            self.address[3],
            self.address[4],
            self.address[5],
        )
    }
}

#[derive(Copy, Clone, PartialOrd, PartialEq)]
pub enum OnWakeOnLan {
    InvokeInterrupt,
    Ignore,
}

#[derive(Copy, Clone, PartialOrd, PartialEq)]
pub enum OnPingRequest {
    Respond,
    Ignore,
}

/// Use [TransmissionMode::PPoE] when talking
/// to an ADSL modem. Otherwise use [TransmissionMode::Ethernet]
#[derive(Copy, Clone, PartialOrd, PartialEq)]
pub enum ConnectionType {
    PPoE,
    Ethernet,
}

#[derive(Copy, Clone, PartialOrd, PartialEq)]
pub enum ArpResponses {
    Cache,
    DropAfterUse,
}

pub struct UninitializedSocket(Socket);
pub struct UdpSocket(Socket);

pub struct W5500<'a> {
    chip_select: &'a mut OutputPin,
    sockets: u8, // each bit represents whether the corresponding socket is available for take
}

impl<'b, 'a: 'b> W5500<'a> {
    fn new(chip_select: &mut OutputPin) -> W5500 {
        W5500 {
            chip_select,
            sockets: 0xFF,
        }
    }

    pub fn with_initialisation<'c, E>(
        chip_select: &'a mut OutputPin,
        spi: &'c mut FullDuplex<u8, Error = E>,
        wol: OnWakeOnLan,
        ping: OnPingRequest,
        mode: ConnectionType,
        arp: ArpResponses,
    ) -> Result<Self, E> {
        let mut w5500 = Self::new(chip_select);
        {
            let mut w5500_active = w5500.activate(spi)?;
            unsafe {
                w5500_active.reset()?;
            }
            w5500_active.update_operation_mode(wol, ping, mode, arp)?;
        }
        Ok(w5500)
    }

    pub fn take_socket(&mut self, socket: Socket) -> Option<UninitializedSocket> {
        let mask = (0x01 << socket.number());
        if self.sockets & mask == mask {
            self.sockets &= !mask;
            Some(UninitializedSocket(socket))
        } else {
            None
        }
    }

    pub fn activate<'c, E>(
        &'b mut self,
        spi: &'c mut FullDuplex<u8, Error = E>,
    ) -> Result<ActiveW5500<'b, 'a, 'c, E>, E> {
        Ok(ActiveW5500(self, spi))
    }
}

pub struct ActiveW5500<'a, 'b: 'a, 'c, E>(&'a mut W5500<'b>, &'c mut FullDuplex<u8, Error = E>);

impl<E> ActiveW5500<'_, '_, '_, E> {
    pub fn take_socket(&mut self, socket: Socket) -> Option<UninitializedSocket> {
        self.0.take_socket(socket)
    }

    pub fn update_operation_mode(
        &mut self,
        wol: OnWakeOnLan,
        ping: OnPingRequest,
        mode: ConnectionType,
        arp: ArpResponses,
    ) -> Result<(), E> {
        let mut value = 0x00;

        if let OnWakeOnLan::InvokeInterrupt = wol {
            value |= (1 << 5);
        }

        if let OnPingRequest::Ignore = ping {
            value |= (1 << 4);
        }

        if let ConnectionType::PPoE = mode {
            value |= (1 << 3);
        }

        if let ArpResponses::DropAfterUse = arp {
            value |= (1 << 1);
        }

        self.write_to(Register::CommonRegister(0x00_00_u16), &[value])
    }

    pub fn set_gateway(&mut self, gateway: IpAddress) -> Result<(), E> {
        self.write_to(Register::CommonRegister(0x00_01_u16), &gateway.address)
    }

    pub fn set_subnet(&mut self, subnet: IpAddress) -> Result<(), E> {
        self.write_to(Register::CommonRegister(0x00_05_u16), &subnet.address)
    }

    pub fn set_mac(&mut self, mac: MacAddress) -> Result<(), E> {
        self.write_to(Register::CommonRegister(0x00_09_u16), &mac.address)
    }

    pub fn set_ip(&mut self, ip: IpAddress) -> Result<(), E> {
        self.write_to(Register::CommonRegister(0x00_0F_u16), &ip.address)
    }

    pub fn read_ip(&mut self, register: Register) -> Result<IpAddress, E> {
        let mut ip = IpAddress::default();
        self.read_from(register, &mut ip.address)?;
        Ok(ip)
    }

    /// This is unsafe because it cannot set taken sockets back to be uninitialized
    /// It assumes, none of the old sockets will used anymore. Otherwise that socket
    /// will have undefined behavior.
    pub unsafe fn reset(&mut self) -> Result<(), E> {
        self.write_to(
            Register::CommonRegister(0x00_00_u16),
            &[
                0b1000_0000, // Mode Register (force reset)
            ],
        )?;
        self.0.sockets = 0xFF;
        Ok(())
    }

    fn is_interrupt_set(&mut self, socket: Socket, interrupt: Interrupt) -> Result<bool, E> {
        let mut state = [0u8; 1];
        self.read_from(socket.at(SocketRegister::Interrupt), &mut state)?;
        Ok(state[0] & interrupt as u8 != 0)
    }

    pub fn reset_interrupt(&mut self, socket: Socket, interrupt: Interrupt) -> Result<(), E> {
        self.write_to(socket.at(SocketRegister::Interrupt), &[interrupt as u8])
    }

    fn read_u8(&mut self, register: Register) -> Result<u8, E> {
        let mut buffer = [0u8; 1];
        self.read_from(register, &mut buffer)?;
        Ok(buffer[0])
    }

    fn read_u16(&mut self, register: Register) -> Result<u16, E> {
        let mut buffer = [0u8; 2];
        self.read_from(register, &mut buffer)?;
        Ok(BigEndian::read_u16(&buffer))
    }

    fn read_from(&mut self, register: Register, target: &mut [u8]) -> Result<(), E> {
        self.chip_select();
        let mut request = [
            0_u8,
            0_u8,
            register.control_byte() | COMMAND_READ | VARIABLE_DATA_LENGTH,
        ];
        BigEndian::write_u16(&mut request[..2], register.address());
        let result = self.write_bytes(&request)
            .and_then(|_| self.read_bytes(target));
        self.chip_deselect();
        result
    }

    fn read_bytes(&mut self, bytes: &mut [u8]) -> Result<(), E> {
        for byte in bytes {
            *byte = self.read()?;
        }
        Ok(())
    }

    fn read(&mut self) -> Result<u8, E> {
        block!(self.1.send(0x00))?;
        block!(self.1.read())
    }

    fn write_u8(&mut self, register: Register, value: u8) -> Result<(), E> {
        self.write_to(register, &[value])
    }

    fn write_u16(&mut self, register: Register, value: u16) -> Result<(), E> {
        let mut data = [0u8; 2];
        BigEndian::write_u16(&mut data, value);
        self.write_to(register, &data)
    }

    fn write_to(&mut self, register: Register, data: &[u8]) -> Result<(), E> {
        self.chip_select();
        let mut request = [
            0_u8,
            0_u8,
            register.control_byte() | COMMAND_WRITE | VARIABLE_DATA_LENGTH,
        ];
        BigEndian::write_u16(&mut request[..2], register.address());
        let result = self.write_bytes(&request)
            .and_then(|_| self.write_bytes(data));
        self.chip_deselect();
        result
    }

    fn write_bytes(&mut self, bytes: &[u8]) -> Result<(), E> {
        for b in bytes {
            self.write(*b)?;
        }
        Ok(())
    }

    fn write(&mut self, byte: u8) -> Result<(), E> {
        block!(self.1.send(byte))?;
        block!(self.1.read())?;
        Ok(())
    }

    fn chip_select(&mut self) {
        self.0.chip_select.set_low()
    }

    fn chip_deselect(&mut self) {
        self.0.chip_select.set_high()
    }
}

pub trait IntoUdpSocket<E> {
    fn try_into_udp_server_socket(self, port: u16) -> Result<UdpSocket, E>
    where
        Self: Sized;
}

impl<E> IntoUdpSocket<UninitializedSocket>
    for (&mut ActiveW5500<'_, '_, '_, E>, UninitializedSocket)
{
    fn try_into_udp_server_socket(self, port: u16) -> Result<UdpSocket, UninitializedSocket> {
        let socket = (self.1).0;
        (|| {
            self.0.reset_interrupt(socket, Interrupt::SendOk)?;

            self.0
                .write_u16(socket.at(SocketRegister::LocalPort), port)?;
            self.0.write_to(
                socket.at(SocketRegister::Mode),
                &[
                    Protocol::UDP as u8,       // Socket Mode Register
                    SocketCommand::Open as u8, // Socket Command Register
                ],
            )?;
            Ok(UdpSocket(socket))
        })()
        .map_err(|_: E| UninitializedSocket(socket))
    }
}

pub trait Udp<E> {
    fn receive(&mut self, target_buffer: &mut [u8]) -> Result<Option<(IpAddress, u16, usize)>, E>;
    fn blocking_send(
        &mut self,
        host: &IpAddress,
        host_port: u16,
        data: &[u8],
    ) -> Result<(), E>;
}

impl<E> Udp<E> for (&mut ActiveW5500<'_, '_, '_, E>, &UdpSocket) {
    fn receive(&mut self, destination: &mut [u8]) -> Result<Option<(IpAddress, u16, usize)>, E> {
        let (w5500, UdpSocket(socket)) = self;

        if w5500.read_u8(socket.at(SocketRegister::InterruptMask))? & 0x04 == 0 {
            return Ok(None);
        }

        let receive_size = loop {
            let s0 = w5500.read_u16(socket.at(SocketRegister::RxReceivedSize))?;
            let s1 = w5500.read_u16(socket.at(SocketRegister::RxReceivedSize))?;
            if s0 == s1 {
                break s0 as usize;
            }
        };
        if receive_size >= 8 {
            let read_pointer = w5500.read_u16(socket.at(SocketRegister::RxReadPointer))?;

            // |<-- read_pointer                                read_pointer + received_size -->|
            // |Destination IP Address | Destination Port | Byte Size of DATA | Actual DATA ... |
            // |   --- 4 Bytes ---     |  --- 2 Bytes --- |  --- 2 Bytes ---  |      ....       |

            let ip = w5500.read_ip(socket.rx_register_at(read_pointer))?;
            let port = w5500.read_u16(socket.rx_register_at(read_pointer + 4))?;
            let data_length = destination
                .len()
                .min(w5500.read_u16(socket.rx_register_at(read_pointer + 6))? as usize);

            w5500.read_from(
                socket.rx_register_at(read_pointer + 8),
                &mut destination[..data_length],
            )?;

            // self.read_from(socket.register_at(0x00_0C), &mut ip.address)?;
            // self.read_u16(socket.register_at(0x00_10))?;

            // reset
            w5500.write_u16(
                socket.at(SocketRegister::RxReadPointer),
                read_pointer + receive_size as u16,
            )?;
            w5500.write_u8(
                socket.at(SocketRegister::Command),
                SocketCommand::Recv as u8,
            )?;

            Ok(Some((ip, port, data_length)))
        } else {
            Ok(None)
        }
    }

    fn blocking_send(
        &mut self,
        host: &IpAddress,
        host_port: u16,
        data: &[u8],
    ) -> Result<(), E> {
        let (w5500, UdpSocket(socket)) = self;

        {
            let local_port = w5500.read_u16(socket.at(SocketRegister::LocalPort))?;
            let local_port = u16_to_be_bytes(local_port);
            let host_port = u16_to_be_bytes(host_port);

            w5500.write_to(
                socket.at(SocketRegister::LocalPort),
                &[
                    local_port[0],
                    local_port[1], // local port u16
                    0x00,
                    0x00,
                    0x00,
                    0x00,
                    0x00,
                    0x00, // destination mac
                    host.address[0],
                    host.address[1],
                    host.address[2],
                    host.address[3], // target IP
                    host_port[0],
                    host_port[1], // destination port (5354)
                ],
            )?;
        }

        let data_length = data.len() as u16;
        {
            let data_length = u16_to_be_bytes(data_length);

            // TODO why write [0x00, 0x00] at TxReadPointer at all?
            // TODO Is TxWritePointer not sufficient enough?
            w5500.write_to(
                socket.at(SocketRegister::TxReadPointer),
                &[0x00, 0x00, data_length[0], data_length[1]],
            );
        }

        w5500.write_to(
            socket.tx_register_at(0x00_00),
            &data[..data_length as usize],
        )?;

        w5500.write_to(
            socket.at(SocketRegister::Command),
            &[SocketCommand::Send as u8],
        )?;

        for _ in 0..0xFFFF {
            // wait until sent
            if w5500.is_interrupt_set(*socket, Interrupt::SendOk)? {
                w5500.reset_interrupt(*socket, Interrupt::SendOk)?;
                break;
            }
        }
        // restore listen state
        /*
        self.network
            .listen_udp(self.spi, SOCKET_UDP, SOCKET_UDP_PORT)
        */
        w5500.write_to(
            socket.at(SocketRegister::Mode),
            &[
                Protocol::UDP as u8,       // Socket Mode Register
                SocketCommand::Open as u8, // Socket Command Register
            ],
        )?;
        Ok(())
    }
}


fn u16_to_be_bytes(u16: u16) -> [u8; 2] {
    let mut bytes = [0u8; 2];
    BigEndian::write_u16(&mut bytes, u16);
    bytes
}

#[repr(u8)]
#[derive(Copy, Clone, PartialEq, Debug)]
pub enum SocketRegister {
    Mode = 0x0000,
    Command = 0x0001,
    Interrupt = 0x0002,
    Status = 0x0003,
    LocalPort = 0x0004,
    DestinationMac = 0x0006,
    DestinationIp = 0x000C,
    DestinationPort = 0x0010,
    MaxSegmentSize = 0x0012,
    // Reserved 0x0014
    TypeOfService = 0x0015,
    TimeToLive = 0x0016,
    // Reserved 0x0017 - 0x001D
    ReceiveBuffer = 0x001E,
    TransmitBuffer = 0x001F,
    TxFreeSize = 0x0020,
    TxReadPointer = 0x0022,
    TxWritePointer = 0x0024,
    RxReceivedSize = 0x0026,
    RxReadPointer = 0x0028,
    RxWritePointer = 0x002A,
    InterruptMask = 0x002C,
    FragmentOffset = 0x002D,
    KeepAliveTimer = 0x002F,
    // Reserved 0x0030 - 0xFFFF
}

#[repr(u8)]
#[derive(Copy, Clone, PartialEq, Debug)]
pub enum Interrupt {
    SendOk = 1 << 4,
    Timeout = 1 << 3,
    Received = 1 << 2,
    Disconnected = 1 << 1,
    Connected = 1, // 1 << 0
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
    pub fn number(self) -> usize {
        match self {
            Socket::Socket0 => 0,
            Socket::Socket1 => 1,
            Socket::Socket2 => 2,
            Socket::Socket3 => 3,
            Socket::Socket4 => 4,
            Socket::Socket5 => 5,
            Socket::Socket6 => 6,
            Socket::Socket7 => 7,
        }
    }

    fn tx_register_at(self, address: u16) -> Register {
        match self {
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

    fn rx_register_at(self, address: u16) -> Register {
        match self {
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

    fn register_at(self, address: u16) -> Register {
        match self {
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

    fn at(self, register: SocketRegister) -> Register {
        self.register_at(register as u16)
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
    fn control_byte(self) -> u8 {
        #[allow(clippy::inconsistent_digit_grouping)]
        match self {
            Register::CommonRegister(_) => 0b00000_000,

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

    fn address(self) -> u16 {
        match self {
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
