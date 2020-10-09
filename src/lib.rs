#![no_std]
#![deny(intra_doc_link_resolution_failure)]

use bit_field::BitField;

mod nal;
pub mod net;
pub use nal::Interface;

pub use embedded_nal::Ipv4Addr;
pub use net::MacAddress;

use embedded_hal::blocking::spi::{Transfer, Write};
use embedded_hal::digital::v2::OutputPin;

use core::convert::TryFrom;
use num_enum::{IntoPrimitive, TryFromPrimitive};

const COMMAND_READ: u8 = 0x00 << 2;
const COMMAND_WRITE: u8 = 0x01 << 2;

const VARIABLE_DATA_LENGTH: u8 = 0b_00;
#[allow(unused)]
const FIXED_DATA_LENGTH_1_BYTE: u8 = 0b_01;
#[allow(unused)]
const FIXED_DATA_LENGTH_2_BYTES: u8 = 0b_10;
#[allow(unused)]
const FIXED_DATA_LENGTH_4_BYTES: u8 = 0b_11;

/// Error enum that represents the union between SPI hardware errors and digital IO pin errors.
/// Returned as an Error type by many [`ActiveW5500`] operations that talk to the chip
#[derive(Copy, Clone, Debug)]
pub enum Error<SpiError, ChipSelectError> {
    Spi(SpiError),
    ChipSelect(ChipSelectError),
    Exhausted,
    NotReady,
    Unsupported,
}

#[derive(TryFromPrimitive)]
#[repr(u8)]
enum SocketState {
    Closed = 0x00,
    Init = 0x13,
    Listen = 0x14,
    Established = 0x17,
    CloseWait = 0x1c,
    Udp = 0x22,
    MacRaw = 0x42,
}

/// Settings for wake on LAN.  Allows the W5500 to optionally emit an interrupt upon receiving a
/// WOL magic packet.
#[derive(Copy, Clone, PartialOrd, PartialEq)]
pub enum OnWakeOnLan {
    InvokeInterrupt,
    Ignore,
}

/// Settings for ping.  Allows the W5500 to respond to or ignore network ping requests.
#[derive(Copy, Clone, PartialOrd, PartialEq)]
pub enum OnPingRequest {
    Respond,
    Ignore,
}

/// Use [`ConnectionType::PPoE`] when talking
/// to an ADSL modem. Otherwise use [`ConnectionType::Ethernet`]
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

/// PHY operation mode.
#[derive(Debug, Copy, Clone, Eq, PartialEq, Ord, PartialOrd)]
#[repr(u8)]
pub enum OperationMode {
    /// 10BT half-duplex. Auto-negotiation disabled.
    HalfDuplex10bt = 0b000,
    /// 10BT full-duplex. Auto-negotiation disabled.
    FullDuplex10bt = 0b001,
    /// 100BT half-duplex. Auto-negotiation disabled.
    HalfDuplex100bt = 0b010,
    /// 100BT full-duplex. Auto-negotiation disabled.
    FullDuplex100bt = 0b011,
    /// 100BT half-duplex. Auto-negotiation enabled.
    HalfDuplex100btAuto = 0b100,
    /// Power down mode.
    PowerDown = 0b110,
    /// All capable. Auto-negotiation enabled.
    Auto = 0b111,
}

impl From<OperationMode> for u8 {
    fn from(val: OperationMode) -> u8 {
        val as u8
    }
}

impl Default for OperationMode {
    fn default() -> OperationMode {
        OperationMode::Auto
    }
}

/// PHY speed status.
#[derive(Debug, Copy, Clone, Eq, PartialEq, Ord, PartialOrd)]
#[repr(u8)]
pub enum SpeedStatus {
    /// 10Mbps based.
    Mbps10 = 0,
    /// 100Mbps based.
    Mbps100 = 1,
}

/// PHY duplex status.
#[derive(Debug, Copy, Clone, Eq, PartialEq, Ord, PartialOrd)]
#[repr(u8)]
pub enum DuplexStatus {
    /// Half duplex.
    HalfDuplex = 0,
    /// Full duplex.
    FullDuplex = 1,
}

/// PHY configuration register.
///
/// Used for:
/// * PHY reset.
/// * PHY operation modes.
/// * PHY status.
#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub struct PhyCfg(u8);

impl PhyCfg {
    // Link status bit position.
    const LNK_POS: u8 = 0;
    // Speed status bit position.
    const SPD_POS: u8 = 1;
    // Duplex status bit position.
    const DPX_POS: u8 = 2;
    // Operation mode bit position.
    const OPMDC_POS: u8 = 3;
    // Configure PHY opeartion mode bit position.
    // const OPMD_POS: u8 = 6;
    // Reset bit position.
    // const RST_POS: u8 = 7;

    /// PHY link status.
    ///
    /// `true` if the link is up, `false` if the link is down.
    pub fn link_up(&self) -> bool {
        self.0 & (1 << PhyCfg::LNK_POS) != 0
    }

    /// PHY speed status.
    pub fn speed(&self) -> SpeedStatus {
        if self.0 & (1 << PhyCfg::SPD_POS) == 0 {
            SpeedStatus::Mbps10
        } else {
            SpeedStatus::Mbps100
        }
    }

    /// PHY duplex status.
    pub fn duplex(&self) -> DuplexStatus {
        if self.0 & (1 << PhyCfg::DPX_POS) == 0 {
            DuplexStatus::HalfDuplex
        } else {
            DuplexStatus::FullDuplex
        }
    }

    /// PHY operation mode.
    pub fn operation_mode(&self) -> OperationMode {
        match (self.0 >> PhyCfg::OPMDC_POS) & 0b111u8 {
            0b000 => OperationMode::HalfDuplex10bt,
            0b001 => OperationMode::FullDuplex10bt,
            0b010 => OperationMode::HalfDuplex100bt,
            0b011 => OperationMode::FullDuplex100bt,
            0b100 => OperationMode::HalfDuplex100btAuto,
            0b110 => OperationMode::PowerDown,
            0b111 => OperationMode::Auto,
            _ => unreachable!(),
        }
    }
}

impl core::convert::From<u8> for PhyCfg {
    fn from(val: u8) -> Self {
        PhyCfg(val)
    }
}

pub struct TcpSocket(Socket);

/// The first level of instantiating communication with the W5500 device. This type is not used
/// for communication, but to keep track of the state of the device. Calling [`W5500::activate`]
/// will return an [`ActiveW5500`] which can be used to communicate with the device. This
/// allows the SPI-Bus to be used for other devices while not being activated without loosing
/// the state.
pub struct W5500<CS: OutputPin, SPI: Transfer<u8> + Write<u8>> {
    chip_select: CS,
    spi: SPI,
    /// each bit represents whether the corresponding socket is available for take
    sockets: u8,

    ephemeral_port: u16,
}

impl<CSE, SPIE, CS, SPI> W5500<CS, SPI>
where
    SPI: Transfer<u8, Error = SPIE> + Write<u8, Error = SPIE>,
    CS: OutputPin<Error = CSE>,
{
    fn get_ephemeral_port(&mut self) -> u16 {
        let current_port = self.ephemeral_port.clone();
        let (next, wrap) = self.ephemeral_port.overflowing_add(1);
        self.ephemeral_port = if wrap { 49152 } else { next };

        current_port
    }

    /// Creates a new instance and initializes the device accordingly to the parameters.
    pub fn new(
        spi: SPI,
        cs: CS,
        wol: OnWakeOnLan,
        ping: OnPingRequest,
        mode: ConnectionType,
        arp: ArpResponses,
    ) -> Result<Self, Error<SPIE, CSE>> {
        let mut w5500 = W5500 {
            chip_select: cs,
            spi,
            sockets: 0xFF,
            ephemeral_port: 49152,
        };

        w5500.reset()?;
        w5500.set_operation_mode(wol, ping, mode, arp)?;

        Ok(w5500)
    }

    /// Attempt to take a socket for use.
    fn take_socket(&mut self) -> Option<Socket> {
        for index in 0..8 {
            if self.sockets.get_bit(index) {
                self.sockets.set_bit(index, false);
                return Some(Socket::try_from(index as u8).unwrap());
            }
        }

        None
    }

    fn return_socket(&mut self, socket: Socket) {
        self.sockets.set_bit(socket as usize, true);
    }

    /// Read the PHY configuration register (PHYCFGR).
    pub fn get_phy_cfg(&mut self) -> Result<PhyCfg, Error<SPIE, CSE>> {
        Ok(self.read_u8(Register::CommonRegister(0x00_2E_u16))?.into())
    }

    /// Set up the basic configuration of the W5500 chip
    fn set_operation_mode(
        &mut self,
        wol: OnWakeOnLan,
        ping: OnPingRequest,
        mode: ConnectionType,
        arp: ArpResponses,
    ) -> Result<(), Error<SPIE, CSE>> {
        let mut value = 0x00;

        if let OnWakeOnLan::InvokeInterrupt = wol {
            value |= 1 << 5;
        }

        if let OnPingRequest::Ignore = ping {
            value |= 1 << 4;
        }

        if let ConnectionType::PPoE = mode {
            value |= 1 << 3;
        }

        if let ArpResponses::DropAfterUse = arp {
            value |= 1 << 1;
        }

        self.write(Register::CommonRegister(0x00_00_u16), &[value])
    }

    /// Sets the IP address of the network gateway (your router's address)
    pub fn set_gateway(&mut self, gateway: Ipv4Addr) -> Result<(), Error<SPIE, CSE>> {
        self.write(Register::CommonRegister(0x00_01_u16), &gateway.octets())
    }

    /// Sets the subnet on the network (for example 255.255.255.0 for /24 subnets)
    pub fn set_subnet(&mut self, subnet: Ipv4Addr) -> Result<(), Error<SPIE, CSE>> {
        self.write(Register::CommonRegister(0x00_05_u16), &subnet.octets())
    }

    /// Sets the MAC address of the W5500 device on the network.
    /// Consider using freely available private/locally administered mac addresses that match the
    /// following hex pattern:
    ///
    /// ```code
    ///  x2-xx-xx-xx-xx-xx
    ///  x6-xx-xx-xx-xx-xx
    ///  xA-xx-xx-xx-xx-xx
    ///  xE-xx-xx-xx-xx-xx
    /// ```
    ///
    /// "Universally administered and locally administered addresses are distinguished by setting
    /// the second-least-significant bit of the first octet of the address" [Wikipedia](https://en.wikipedia.org/wiki/MAC_address#Universal_vs._local)
    ///
    pub fn set_mac(&mut self, mac: MacAddress) -> Result<(), Error<SPIE, CSE>> {
        self.write(Register::CommonRegister(0x00_09_u16), &mac.octets)
    }

    pub fn get_mac(&mut self) -> Result<MacAddress, Error<SPIE, CSE>> {
        let mut octets: [u8; 6] = [0; 6];
        self.read(Register::CommonRegister(0x00_09_u16), &mut octets)?;
        Ok(MacAddress::from_bytes(octets))
    }

    pub fn get_ip(&mut self) -> Result<Ipv4Addr, Error<SPIE, CSE>> {
        let mut octets: [u8; 4] = [0; 4];
        self.read(Register::CommonRegister(0x00_0F_u16), &mut octets)?;
        Ok(Ipv4Addr::new(octets[0], octets[1], octets[2], octets[3]))
    }

    /// Sets the IP address of the W5500 device.  Must be within the range and permitted by the
    /// gateway or the device will not be accessible.
    pub fn set_ip(&mut self, ip: Ipv4Addr) -> Result<(), Error<SPIE, CSE>> {
        self.write(Register::CommonRegister(0x00_0F_u16), &ip.octets())
    }

    fn reset(&mut self) -> Result<(), Error<SPIE, CSE>> {
        self.write(
            Register::CommonRegister(0x00_00_u16),
            &[
                0b1000_0000, // Mode Register (force reset)
            ],
        )?;

        // Wait for the RST bit to de-assert to indicate software reset complete.
        while self.read_u8(Register::CommonRegister(0x0000_u16))?.get_bit(7) {};

        self.sockets = 0xFF;
        Ok(())
    }

    /// TODO document
    fn interrupt_is_set(
        &mut self,
        socket: Socket,
        interrupt: Interrupt,
    ) -> Result<bool, Error<SPIE, CSE>> {
        let mut state = [0u8; 1];
        self.read(socket.at(SocketRegister::Interrupt), &mut state)?;
        Ok(state[0] & (interrupt as u8) != 0)
    }

    /// TODO document
    pub fn reset_interrupt(
        &mut self,
        socket: Socket,
        interrupt: Interrupt,
    ) -> Result<(), Error<SPIE, CSE>> {
        self.write(socket.at(SocketRegister::Interrupt), &[interrupt as u8])
    }

    /// Reads one byte from the given [`Register`] as a u8
    fn read_u8(&mut self, register: Register) -> Result<u8, Error<SPIE, CSE>> {
        let mut buffer = [0u8; 1];
        self.read(register, &mut buffer)?;
        Ok(buffer[0])
    }

    /// Reads two bytes from the given [`Register`] as a u16
    fn read_u16(&mut self, register: Register) -> Result<u16, Error<SPIE, CSE>> {
        let mut buffer = [0u8; 2];
        self.read(register, &mut buffer)?;
        Ok(u16::from_be_bytes(buffer))
    }

    /// Write a single u8 byte to the given [`Register`]
    fn write_u8(&mut self, register: Register, value: u8) -> Result<(), Error<SPIE, CSE>> {
        self.write(register, &[value])
    }

    /// Write a u16 as two bytes o the given [`Register`]
    fn write_u16(&mut self, register: Register, value: u16) -> Result<(), Error<SPIE, CSE>> {
        self.write(register, &value.to_be_bytes())
    }

    /// Reads enough bytes from the given [`Register`] address onward to fill the `target` u8 slice
    fn read(&mut self, register: Register, target: &mut [u8]) -> Result<(), Error<SPIE, CSE>> {
        self.chip_select
            .set_low()
            .map_err(|e| Error::<SPIE, CSE>::ChipSelect(e))?;

        // Write the address phase
        let address = register.address();
        self.spi
            .write(&address.to_be_bytes())
            .map_err(|e| Error::Spi::<SPIE, CSE>(e))?;

        // Write the control byte
        let control = register.control_byte() | COMMAND_READ | VARIABLE_DATA_LENGTH;
        self.spi
            .write(&[control])
            .map_err(|e| Error::<SPIE, CSE>::Spi(e))?;

        // Transact the data.
        self.spi
            .transfer(target)
            .map_err(|e| Error::<SPIE, CSE>::Spi(e))?;

        self.chip_select
            .set_high()
            .map_err(|e| Error::<SPIE, CSE>::ChipSelect(e))?;

        Ok(())
    }

    /// Write a slice of u8 bytes to the given [`Register`]
    fn write(&mut self, register: Register, data: &[u8]) -> Result<(), Error<SPIE, CSE>> {
        self.chip_select
            .set_low()
            .map_err(|e| Error::<SPIE, CSE>::ChipSelect(e))?;

        // Write the address phase
        let address = register.address();
        self.spi
            .write(&address.to_be_bytes())
            .map_err(|e| Error::Spi::<SPIE, CSE>(e))?;

        // Write the control byte
        let control = register.control_byte() | COMMAND_WRITE | VARIABLE_DATA_LENGTH;
        self.spi
            .write(&[control])
            .map_err(|e| Error::<SPIE, CSE>::Spi(e))?;

        // Write the data.
        self.spi
            .write(&data)
            .map_err(|e| Error::<SPIE, CSE>::Spi(e))?;

        self.chip_select
            .set_high()
            .map_err(|e| Error::<SPIE, CSE>::ChipSelect(e))?;

        Ok(())
    }

    pub fn open_tcp(&mut self) -> Result<TcpSocket, Error<SPIE, CSE>> {
        let socket = self.take_socket().ok_or(Error::<SPIE, CSE>::Exhausted)?;

        self.write_u8(socket.at(SocketRegister::Mode), Protocol::TCP as u8)
            .or_else(|e| {
                self.return_socket(socket);
                Err(e)
            })?;

        // Open the socket.
        self.write_u8(
            socket.at(SocketRegister::Command),
            SocketCommand::Open as u8,
        )
        .or_else(|e| {
            self.return_socket(socket);
            Err(e)
        })?;

        // Wait for the socket to enter the INIT state.
        loop {
            let status = self
                .read_u8(socket.at(SocketRegister::Status))
                .or_else(|e| {
                    self.return_socket(socket);
                    Err(e)
                })?;
            if status == SocketState::Init as u8 {
                break;
            }
        }

        Ok(TcpSocket(socket))
    }

    pub fn connect_tcp(
        &mut self,
        socket: TcpSocket,
        remote: Ipv4Addr,
        port: u16,
    ) -> Result<TcpSocket, Error<SPIE, CSE>> {
        // Ensure the socket is open before we attempt to connect it.
        let state = self.read_u8(socket.0.at(SocketRegister::Status))?;
        let socket = match SocketState::try_from(state) {
            Ok(SocketState::Init) => socket,
            _ => {
                self.close(socket)?;
                self.open_tcp()?
            }
        };

        // Set our local port to some ephemeral port.
        let local_port = self.get_ephemeral_port();
        self.write_u16(socket.0.at(SocketRegister::LocalPort), local_port)?;

        // Write the report port and IP
        self.write(socket.0.at(SocketRegister::DestinationIp), &remote.octets())?;

        self.write_u16(socket.0.at(SocketRegister::DestinationPort), port)?;

        // Connect the socket.
        self.write_u8(
            socket.0.at(SocketRegister::Command),
            SocketCommand::Connect as u8,
        )?;

        // Wait for the socket to connect or encounter an error.
        loop {
            match SocketState::try_from(self.read_u8(socket.0.at(SocketRegister::Status))?) {
                Ok(SocketState::Established) => return Ok(socket),

                // The socket is closed if a timeout (ARP or SYN-ACK) or if the TCP socket receives
                // a RST packet. In this case, the client will need to re-attempt to connect.

                // TODO: Due to limitations of the embedded-nal, we currently still return the
                // socket (since we cannot inform the user of the connection failure). The returned
                // socket will not actually be connected.
                Ok(SocketState::Closed) => {
                    // For now, always return an open socket so that the user can re-connect with
                    // it in the future.
                    self.close(socket)?;
                    return self.open_tcp();
                }
                // The socket is still in some transient state. Wait for it to connect or for the
                // connection to fail.
                _ => {}
            }
        }
    }

    pub fn is_connected(&mut self, socket: &TcpSocket) -> Result<bool, Error<SPIE, CSE>> {
        // Read the status register to ensure the connection is established.
        let status = self.read_u8(socket.0.at(SocketRegister::Status))?;
        Ok(status == SocketState::Established as u8)
    }

    pub fn send(&mut self, socket: &TcpSocket, data: &[u8]) -> Result<usize, Error<SPIE, CSE>> {
        if self.is_connected(&socket)? == false {
            return Err(Error::<SPIE, CSE>::NotReady);
        }

        let max_size = self.read_u16(socket.0.at(SocketRegister::TxFreeSize))? as usize;

        let write_data = if data.len() < max_size {
            data
        } else {
            &data[..max_size]
        };

        // Append the data to the write buffer.
        let write_pointer = self.read_u16(socket.0.at(SocketRegister::TxWritePointer))?;
        self.write(socket.0.tx_register_at(write_pointer), write_data)?;

        // Update the writer pointer.
        self.write_u16(
            socket.0.at(SocketRegister::TxWritePointer),
            write_pointer.wrapping_add(write_data.len() as u16),
        )?;

        // Send the data.
        self.write_u8(
            socket.0.at(SocketRegister::Command),
            SocketCommand::Send as u8,
        )?;

        // Wait until the send command completes.
        while self.interrupt_is_set(socket.0, Interrupt::SendOk)? == false {}
        self.reset_interrupt(socket.0, Interrupt::SendOk)?;

        Ok(write_data.len())
    }

    pub fn recv(&mut self, socket: &TcpSocket, data: &mut [u8]) -> Result<usize, Error<SPIE, CSE>> {
        if self.is_connected(&socket)? == false {
            return Err(Error::<SPIE, CSE>::NotReady);
        }

        // Check if we've received data.
        if self.interrupt_is_set(socket.0, Interrupt::Received)? == false {
            return Ok(0);
        }

        let rx_size = loop {
            let s0 = self.read_u16(socket.0.at(SocketRegister::RxReceivedSize))?;
            let s1 = self.read_u16(socket.0.at(SocketRegister::RxReceivedSize))?;
            if s0 == s1 {
                break s0 as usize;
            }
        };

        let read_buffer = if rx_size > data.len() {
            data
        } else {
            &mut data[..rx_size]
        };

        // Read from the RX ring buffer.
        let read_pointer = self.read_u16(socket.0.at(SocketRegister::RxReadPointer))?;
        self.read(socket.0.rx_register_at(read_pointer), read_buffer)?;
        self.write_u16(
            socket.0.at(SocketRegister::RxReadPointer),
            read_pointer.wrapping_add(read_buffer.len() as u16),
        )?;

        // Register the reception as complete.
        self.write_u8(
            socket.0.at(SocketRegister::Command),
            SocketCommand::Recv as u8,
        )?;
        self.reset_interrupt(socket.0, Interrupt::Received)?;

        Ok(read_buffer.len())
    }

    pub fn disconnect(&mut self, socket: &TcpSocket) -> Result<(), Error<SPIE, CSE>> {
        self.write_u8(
            socket.0.at(SocketRegister::Command),
            SocketCommand::Disconnect as u8,
        )
    }

    pub fn close(&mut self, socket: TcpSocket) -> Result<(), Error<SPIE, CSE>> {
        self.write_u8(
            socket.0.at(SocketRegister::Command),
            SocketCommand::Close as u8,
        )?;
        self.return_socket(socket.0);
        Ok(())
    }
}

/// Offset addresses in each socket register
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

/// Interrupt state bits
#[repr(u8)]
#[derive(Copy, Clone, PartialEq, Debug)]
pub enum Interrupt {
    SendOk = 1 << 4,
    Timeout = 1 << 3,
    Received = 1 << 2,
    Disconnected = 1 << 1,
    Connected = 1, // 1 << 0
}

/// Register protocol mode bits
#[repr(u8)]
#[derive(Copy, Clone, PartialEq, Debug)]
pub enum Protocol {
    TCP = 0b0001,
    UDP = 0b0010,
    MACRAW = 0b0100,
}

/// Bits for socket commands
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

/// Identifiers for each socket on the W5500
#[derive(Copy, Clone, PartialEq, PartialOrd, Debug, IntoPrimitive, TryFromPrimitive)]
#[repr(u8)]
pub enum Socket {
    Zero = 0,
    One = 1,
    Two = 2,
    Three = 3,
    Four = 4,
    Five = 5,
    Six = 6,
    Seven = 7,
}

impl Socket {
    /// Returns the register address for a socket instance's TX
    fn tx_register_at(self, address: u16) -> Register {
        match self {
            Socket::Zero => Register::Socket0TxBuffer(address),
            Socket::One => Register::Socket1TxBuffer(address),
            Socket::Two => Register::Socket2TxBuffer(address),
            Socket::Three => Register::Socket3TxBuffer(address),
            Socket::Four => Register::Socket4TxBuffer(address),
            Socket::Five => Register::Socket5TxBuffer(address),
            Socket::Six => Register::Socket6TxBuffer(address),
            Socket::Seven => Register::Socket7TxBuffer(address),
        }
    }

    /// Returns the register address for a socket instance's RX
    fn rx_register_at(self, address: u16) -> Register {
        match self {
            Socket::Zero => Register::Socket0RxBuffer(address),
            Socket::One => Register::Socket1RxBuffer(address),
            Socket::Two => Register::Socket2RxBuffer(address),
            Socket::Three => Register::Socket3RxBuffer(address),
            Socket::Four => Register::Socket4RxBuffer(address),
            Socket::Five => Register::Socket5RxBuffer(address),
            Socket::Six => Register::Socket6RxBuffer(address),
            Socket::Seven => Register::Socket7RxBuffer(address),
        }
    }

    /// Returns the register address for a socket instance's register
    fn register_at(self, address: u16) -> Register {
        match self {
            Socket::Zero => Register::Socket0Register(address),
            Socket::One => Register::Socket1Register(address),
            Socket::Two => Register::Socket2Register(address),
            Socket::Three => Register::Socket3Register(address),
            Socket::Four => Register::Socket4Register(address),
            Socket::Five => Register::Socket5Register(address),
            Socket::Six => Register::Socket6Register(address),
            Socket::Seven => Register::Socket7Register(address),
        }
    }

    fn at(self, register: SocketRegister) -> Register {
        self.register_at(register as u16)
    }
}

/// Chip register names
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
    /// Gets the control bits to identify any given register
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

    /// Returns the associated address as a u16
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
