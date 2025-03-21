#![allow(clippy::inconsistent_digit_grouping, clippy::unusual_byte_groupings)]

// TODO change from u8 to a custom struct implementing a trait.
pub const COMMON: u8 = 0;
pub mod common {
    use bit_field::BitArray;

    pub const MODE: u16 = 0x0;

    /// Register: GAR (Gateway IP Address Register) [R/W] [0x0001 – 0x0004] [0x00]
    pub const GATEWAY: u16 = 0x01;

    /// Register: SUBR (Subnet Mask Register) [R/W] [0x0005 – 0x0008] [0x00]
    pub const SUBNET_MASK: u16 = 0x05;

    /// Register: SHAR (Source Hardware Address Register) [R/W] [0x0009 – 0x000E] [0x00]
    pub const MAC: u16 = 0x09;

    /// Register: SIPR (Source IP Address Register) [R/W] [0x000F – 0x0012] [0x00]
    pub const IP: u16 = 0x0F;

    /// Register: INTLEVEL (Interrupt Low Level Timer Register) [R/W] [0x0013 – 0x0014] [0x0000]
    pub const INTERRUPT_TIMER: u16 = 0x13;

    /// Register: SIMR (Socket Interrupt Mask Register) [R/W] [0x0018] [0x00]
    pub const SOCKET_INTERRUPT_MASK: u16 = 0x18;

    /// Register: RTR (Retry Time-value Register) [R/W] [0x0019 – 0x001A] [0x07D0]
    pub const RETRY_TIME: u16 = 0x19;

    /// Register: RCR (Retry Count Register) [R/W] [0x001B] [0x08]
    pub const RETRY_COUNT: u16 = 0x1B;

    pub const PHY_CONFIG: u16 = 0x2E;
    pub const VERSION: u16 = 0x39;

    /// A Retry Time-value
    ///
    /// RTR (Retry Time-value Register) [R/W] [0x0019 – 0x001A] [0x07D0]
    ///
    /// From datasheet:
    ///
    /// RTR configures the retransmission timeout period. The unit of timeout period is
    /// 100us and the default of RTR is ‘0x07D0’ or ‘2000’. And so the default timeout period
    /// is 200ms(100us X 2000).
    /// During the time configured by RTR, W5500 waits for the peer response to the packet
    /// that is transmitted by Sn_CR(CONNECT, DISCON, CLOSE, SEND, SEND_MAC, SEND_KEEP
    /// command). If the peer does not respond within the RTR time, W5500 retransmits the
    /// packet or issues timeout.
    ///
    ///
    /// > Ex) When timeout-period is set as 400ms, RTR = (400ms / 1ms) X 10 = 4000(0x0FA0)
    #[derive(Debug, Copy, Clone, Eq, PartialEq, Ord, PartialOrd)]
    #[cfg_attr(feature = "defmt", derive(defmt::Format))]
    pub struct RetryTime(pub(crate) u16);

    impl RetryTime {
        #[inline]
        pub fn to_u16(&self) -> u16 {
            self.0
        }

        #[inline]
        pub fn to_register(&self) -> [u8; 2] {
            self.0.to_be_bytes()
        }

        #[inline]
        pub fn from_register(register: [u8; 2]) -> Self {
            Self(u16::from_be_bytes(register))
        }

        #[inline]
        pub fn from_millis(milliseconds: u16) -> Self {
            Self(milliseconds * 10)
        }

        #[inline]
        pub fn to_millis(&self) -> u16 {
            self.0 / 10
        }
    }

    impl Default for RetryTime {
        fn default() -> Self {
            Self::from_millis(200)
        }
    }

    #[derive(Default, Debug, Copy, Clone, Eq, PartialEq, Ord, PartialOrd)]
    #[repr(u8)]
    pub enum PhyOperationMode {
        /// 10BT half-duplex. Auto-negotiation disabled.
        HalfDuplex10bt = 0b000_000,
        /// 10BT full-duplex. Auto-negotiation disabled.
        FullDuplex10bt = 0b001_000,
        /// 100BT half-duplex. Auto-negotiation disabled.
        HalfDuplex100bt = 0b010_000,
        /// 100BT full-duplex. Auto-negotiation disabled.
        FullDuplex100bt = 0b011_000,
        /// 100BT half-duplex. Auto-negotiation enabled.
        HalfDuplex100btAuto = 0b100_000,
        /// Power down mode.
        PowerDown = 0b110_000,
        /// All capable. Auto-negotiation enabled.
        #[default]
        Auto = 0b111_000,
    }

    impl From<PhyOperationMode> for u8 {
        fn from(val: PhyOperationMode) -> u8 {
            val as u8
        }
    }

    #[derive(Debug, Copy, Clone, Eq, PartialEq, Ord, PartialOrd)]
    #[repr(u8)]
    pub enum PhySpeedStatus {
        /// 10Mbps based.
        Mbps10 = 0,
        /// 100Mbps based.
        Mbps100 = 1,
    }

    #[derive(Debug, Copy, Clone, Eq, PartialEq, Ord, PartialOrd)]
    #[repr(u8)]
    pub enum PhyDuplexStatus {
        /// Half duplex.
        HalfDuplex = 0,
        /// Full duplex.
        FullDuplex = 1,
    }

    #[derive(Debug, Copy, Clone, Eq, PartialEq)]
    pub struct PhyConfig([u8; 1]);

    impl PhyConfig {
        // Link status bit position.
        const LNK_POS: usize = 0;
        // Speed status bit position.
        const SPD_POS: usize = 1;
        // Duplex status bit position.
        const DPX_POS: usize = 2;
        // Operation mode bit position.
        // const OPMDC_POS = 3;
        // Configure PHY opeartion mode bit position.
        // const OPMD_POS = 6;
        // Reset bit position.
        // const RST_POS = 7;

        /// PHY link status.
        ///
        /// `true` if the link is up, `false` if the link is down.
        pub fn link_up(&self) -> bool {
            self.0.get_bit(Self::LNK_POS)
        }

        /// PHY speed status.
        pub fn speed(&self) -> PhySpeedStatus {
            if !self.0.get_bit(Self::SPD_POS) {
                PhySpeedStatus::Mbps10
            } else {
                PhySpeedStatus::Mbps100
            }
        }

        /// PHY duplex status.
        pub fn duplex(&self) -> PhyDuplexStatus {
            if !self.0.get_bit(Self::DPX_POS) {
                PhyDuplexStatus::HalfDuplex
            } else {
                PhyDuplexStatus::FullDuplex
            }
        }

        /// PHY operation mode.
        pub fn operation_mode(&self) -> PhyOperationMode {
            match self.0[0] & 0b111_000u8 {
                0b000_000 => PhyOperationMode::HalfDuplex10bt,
                0b001_000 => PhyOperationMode::FullDuplex10bt,
                0b010_000 => PhyOperationMode::HalfDuplex100bt,
                0b011_000 => PhyOperationMode::FullDuplex100bt,
                0b100_000 => PhyOperationMode::HalfDuplex100btAuto,
                0b110_000 => PhyOperationMode::PowerDown,
                0b111_000 => PhyOperationMode::Auto,
                _ => unreachable!(),
            }
        }
    }

    impl core::convert::From<u8> for PhyConfig {
        fn from(val: u8) -> Self {
            PhyConfig([val])
        }
    }
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
    use derive_try_from_primitive::TryFromPrimitive;

    /// The Protocol mode
    pub const MODE: u16 = 0x00;

    /// The protocol modes that can be used with the `w5500`
    #[repr(u8)]
    pub enum Protocol {
        Closed = 0b00,
        Tcp = 0b01,
        Udp = 0b10,
        MacRaw = 0b100,
    }

    /// Socket n Command Register
    ///
    /// `Sn_CR`
    pub const COMMAND: u16 = 0x01;

    /// Socket n Commands
    ///
    /// `Sn_CR` register
    #[repr(u8)]
    pub enum Command {
        Open = 0x01,
        /// [Datasheet page 46](https://docs.wiznet.io/img/products/w5500/W5500_ds_v110e.pdf):
        ///
        /// > This is valid only in TCP mode (Sn_MR(P3:P0) = Sn_MR_TCP). In this
        /// > mode, Socket n operates as a ‘TCP server’ and waits for connection-
        /// > request (SYN packet) from any ‘TCP client
        Listen = 0x02,
        Connect = 0x04,
        Discon = 0x08,
        Close = 0x10,
        Send = 0x20,

        /// [Datasheet page 48](https://docs.wiznet.io/img/products/w5500/W5500_ds_v110e.pdf):
        ///
        /// > RECV completes the processing of the received data in Socket n RX
        /// > Buffer by using a RX read pointer register (Sn_RX_RD).
        /// > For more details, refer to Socket n RX Received Size Register
        /// > (Sn_RX_RSR), Socket n RX Write Pointer Register (Sn_RX_WR), and
        /// > Socket n RX Read Pointer Register (Sn_RX_RD).
        Receive = 0x40,
    }

    pub const INTERRUPT: u16 = 0x02;
    /// | 7 | 6 | 5 | 4 | 3 | 2 | 1 | 0 |
    /// | Reserved | Reserved | Reserved | SEND_OK | TIMEOUT | RECV | DISCON | CON |
    ///
    /// | Bit | Symbol | Description |
    /// | 7~5 | Reserved | Reserved |
    /// | 4 | SENDOK | Sn_IR(SENDOK) | Interrupt Mask |
    /// | 3 | TIMEOUT | Sn_IR(TIMEOUT) | Interrupt Mask |
    /// | 2 | RECV | Sn_IR(RECV) | Interrupt Mask |
    /// | 1 | DISCON | Sn_IR(DISCON) | Interrupt Mask |
    /// | 0 | CON | Sn_IR(CON) | Interrupt Mask |
    #[repr(u8)]
    pub enum Interrupt {
        All = 0b11111111u8,
        SendOk = 0b10000u8,
        Timeout = 0b1000u8,

        /// Receive data
        ///
        /// bit 2, symbol `RECV`, `Sn_IR(RECV) Interrupt Mask`
        Receive = 0b100u8,

        /// Disconnect
        ///
        /// bit 1, symbol `DISCON`, `Sn_IR(DISCON) Interrupt Mask`
        Disconnect = 0b10u8,

        /// Connect
        ///
        /// bit 0, symbol `CON`, `Sn_IR(CON) Interrupt Mask`
        Connect = 0b1u8,
    }

    pub const STATUS: u16 = 0x03;

    /// Socket status register
    ///
    /// `W5500 Datasheet Version 1.1.0` page 49:
    ///
    /// > Sn_SR (Socket n Status Register) [R] [0x0003] [0x00]
    ///
    /// - 0x18 SOCK_FIN_WAIT
    /// - 0x1A SOCK_CLOSING
    /// - 0X1B SOCK_TIME_WAIT
    /// > These indicate Socket n is closing.
    /// > These are shown in disconnect-process such as active-close
    /// > and passive-close.
    /// > When Disconnect-process is successfully completed, or
    /// > when timeout occurs, these change to SOCK_CLOSED.
    ///
    #[derive(TryFromPrimitive, Debug, Copy, Clone, PartialEq, Eq)]
    #[repr(u8)]
    pub enum Status {
        Closed = 0x00,
        Init = 0x13,

        /// [Datasheet page 49](https://docs.wiznet.io/img/products/w5500/W5500_ds_v110e.pdf):
        ///
        /// > This indicates Socket n is operating as ‘TCP server’ mode and
        /// > waiting for connection-request (SYN packet) from a peer
        /// > (‘TCP client’).
        Listen = 0x14,
        Established = 0x17,
        CloseWait = 0x1c,
        Udp = 0x22,
        MacRaw = 0x42,

        // Transient states.
        SynSent = 0x15,
        SynRecv = 0x16,
        FinWait = 0x18,
        Closing = 0x1a,
        TimeWait = 0x1b,
        LastAck = 0x1d,
    }

    #[cfg(feature = "defmt")]
    impl defmt::Format for Status {
        fn format(&self, fmt: defmt::Formatter) {
            // Format as hexadecimal.
            defmt::write!(
                fmt,
                "Status::{} ({=u8:#x})",
                defmt::Debug2Format(self),
                *self as u8
            );
        }
    }

    pub const SOURCE_PORT: u16 = 0x04;

    pub const DESTINATION_IP: u16 = 0x0C;

    pub const DESTINATION_PORT: u16 = 0x10;

    pub const RXBUF_SIZE: u16 = 0x1E;

    /// Socket n TX Buffer Size Register
    ///
    /// `Sn_TXBUF_SIZE`
    ///
    /// From datasheet:
    ///
    /// > .. can be configured with 1,2,4,8, and 16 Kbytes.
    /// >
    /// > Although Socket n TX Buffer Block size is initially configured to 2Kbytes, user can
    /// > be re-configure its size using Sn_TXBUF_SIZE. The total sum of Sn_TXBUF_SIZE
    /// > cannot be exceed 16Kbytes. When exceeded, the data transmission error is
    /// > occurred.
    pub const TXBUF_SIZE: u16 = 0x1F;

    /// TX Free Size Register
    ///
    /// `Sn_TX_FSR`
    ///
    /// Socket n TX Free Size
    ///
    /// offset (register)
    /// 0x0020 (Sn_TX_FSR0)
    /// 0x0021 (Sn_TX_FSR1)
    pub const TX_FREE_SIZE: u16 = 0x20;

    /// Socket n TX Read Pointer
    ///
    /// offset (register)
    /// 0x0022 (Sn_TX_RD0)
    /// 0x0023 (Sn_TX_RD1)
    pub const TX_DATA_READ_POINTER: u16 = 0x22;

    /// Socket n TX Write Pointer
    ///
    /// offset (register)
    /// 0x0024 (Sn_TX_WR0)
    /// 0x0025 (Sn_TX_WR1)
    ///
    /// [Datasheet page 54](https://docs.wiznet.io/img/products/w5500/W5500_ds_v110e.pdf):
    ///
    /// > Sn_TX_WR (Socket n TX Write Pointer Register) [R/W] [0x0024-0x0025] [0x0000]
    /// >
    /// > Sn_TX_WR is initialized by OPEN command. However, if Sn_MR(P[3:0]) is TCP
    /// > mode(‘0001’), it is re-initialized while connecting with TCP.
    /// > It should be read or to be updated like as follows.
    /// > 1. Read the starting address for saving the transmitting data.
    /// > 2. Save the transmitting data from the starting address of Socket n TX
    /// >    buffer.
    /// > 3. After saving the transmitting data, update Sn_TX_WR to the
    /// >    increased value as many as transmitting data size. If the increment value
    /// >    exceeds the maximum value 0xFFFF(greater than 0x10000 and the carry
    /// >    bit occurs), then the carry bit is ignored and will automatically update
    /// >    with the lower 16bits value.
    /// > 4. Transmit the saved data in Socket n TX Buffer by using SEND/SEND
    /// >    command
    pub const TX_DATA_WRITE_POINTER: u16 = 0x24;

    /// Socket n Received Size Register
    ///
    /// `Sn_RX_RSR`
    pub const RECEIVED_SIZE: u16 = 0x26;

    pub const RX_DATA_READ_POINTER: u16 = 0x28;

    /// Socket n Interrupt Mask
    ///
    /// offset (register)
    /// 0x002C (Sn_IMR)
    pub const INTERRUPT_MASK: u16 = 0x2C;

    #[cfg(test)]
    mod tests {
        use core::convert::TryFrom;

        use super::Status;

        #[test]
        fn test_status_from_byte() {
            let udp = 0x22_u8;
            let status = Status::try_from(udp).expect("Should parse to Status");
            assert_eq!(status, Status::Udp);
        }
    }
}
