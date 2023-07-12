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

    /// Register: RTR (Retry Time-value Register) [R/W] [0x0019 – 0x001A] [0x07D0]
    pub const RETRY_TIME: u16 = 0x19;

    /// Register: RCR (Retry Count Register) [R/W] [0x001B] [0x08]
    pub const RETRY_COUNT: u16 = 0x1B;

    pub const PHY_CONFIG: u16 = 0x2E;
    pub const VERSION: u16 = 0x39;

    /// The common register Mode.
    ///
    /// It differs from the [`crate::Mode`] in one key aspect - we can also
    /// send a reset value to the w5500, instead of only configuring the settings.
    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    #[cfg_attr(feature = "defmt", derive(defmt::Format))]
    pub enum Mode {
        Reset,
        Mode(crate::Mode),
    }

    impl Mode {
        pub fn to_register(self) -> [u8; 1] {
            [self.to_u8()]
        }

        pub fn to_u8(self) -> u8 {
            match self {
                Mode::Reset => 0b10000000,
                Mode::Mode(mode) => mode.to_u8(),
            }
        }
    }

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

    /// RCR (Retry Count Register) [R/W] [0x001B] [0x08]
    ///
    /// For more details check out the rest of the datasheet documentation on the Retry count.
    ///
    /// From datasheet:
    ///
    /// RCR configures the number of time of retransmission. When retransmission occurs
    /// as many as ‘RCR+1’, Timeout interrupt is issued (Sn_IR[TIMEOUT] = ‘1’).
    ///
    /// > Ex) RCR = 0x0007
    ///
    /// The timeout of W5500 can be configurable with RTR and RCR. W5500 has two kind
    /// timeout such as Address Resolution Protocol (ARP) and TCP retransmission.
    ///
    #[derive(Debug, Copy, Clone, Eq, PartialEq, Ord, PartialOrd)]
    #[cfg_attr(feature = "defmt", derive(defmt::Format))]
    pub struct RetryCount(pub u8);

    impl RetryCount {
        #[inline]
        pub fn to_u8(&self) -> u8 {
            self.0
        }

        #[inline]
        pub fn to_register(&self) -> [u8; 1] {
            self.0.to_be_bytes()
        }

        #[inline]
        pub fn from_register(register: [u8; 1]) -> Self {
            Self(register[0])
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

    #[cfg(test)]
    mod test {
        use super::Mode;

        #[test]
        fn test_mode_register() {
            let reset = Mode::Reset;
            // Bit: 7 Reset (RST) should be 1
            assert_eq!(0b1000_0000, reset.to_u8());

            let ping_respond_and_force_arp = crate::Mode {
                // Bit: 7 Reset (RST) should be 0
                // Bit: 6 reserved
                // Bit: 5 should be 0 - Disable WOL mode
                on_wake_on_lan: crate::OnWakeOnLan::Ignore,
                // Bit: 4 should be 0 - Disable Ping Block Mode
                on_ping_request: crate::OnPingRequest::Respond,
                // Bit: 3 should be 0 - PPoE disabled
                connection_type: crate::ConnectionType::Ethernet,
                // Bit: 2 reserved
                // Bit: 1 should be 0 - Disabled Force ARP
                arp_responses: crate::ArpResponses::Cache,
                // Bit: 0 reserved
            };
            assert_eq!(0b0000_0000, ping_respond_and_force_arp.to_u8());

            let all_enabled = crate::Mode {
                // Bit: 7 Reset (RST) should be 0
                // Bit: 6 reserved
                // Bit: 5 should be 1 - Enable WOL mode
                on_wake_on_lan: crate::OnWakeOnLan::InvokeInterrupt,
                // Bit: 4 should be 0 - Disable Ping Block Mode
                on_ping_request: crate::OnPingRequest::Respond,
                // Bit: 3 should be 1 - PPoE enable
                connection_type: crate::ConnectionType::PPoE,
                // Bit: 2 reserved
                // Bit: 1 should be 1 - Enable Force ARP
                arp_responses: crate::ArpResponses::DropAfterUse,
                // Bit: 0 reserved
            };
            assert_eq!(0b0010_1010, all_enabled.to_u8());
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

    pub const MODE: u16 = 0x00;
    #[repr(u8)]
    pub enum Protocol {
        Closed = 0b00,
        Tcp = 0b01,
        Udp = 0b10,
        MacRaw = 0b100,
    }
    pub const COMMAND: u16 = 0x01;
    #[repr(u8)]
    pub enum Command {
        Open = 0x01,
        Listen = 0x02,
        Connect = 0x04,
        Discon = 0x08,
        Close = 0x10,
        Send = 0x20,
        Receive = 0x40,
    }

    pub const INTERRUPT: u16 = 0x02;
    #[repr(u8)]
    pub enum Interrupt {
        All = 0b11111111u8,
        SendOk = 0b010000u8,
        Timeout = 0b01000u8,
        Receive = 0b00100u8,
    }

    pub const STATUS: u16 = 0x03;
    #[repr(u8)]
    #[derive(TryFromPrimitive)]
    pub enum Status {
        Closed = 0x00,
        Init = 0x13,
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

    pub const SOURCE_PORT: u16 = 0x04;

    pub const DESTINATION_IP: u16 = 0x0C;

    pub const DESTINATION_PORT: u16 = 0x10;

    pub const RXBUF_SIZE: u16 = 0x1E;

    pub const TXBUF_SIZE: u16 = 0x1F;

    pub const TX_FREE_SIZE: u16 = 0x20;

    pub const TX_DATA_READ_POINTER: u16 = 0x22;

    pub const TX_DATA_WRITE_POINTER: u16 = 0x24;

    pub const RECEIVED_SIZE: u16 = 0x26;

    pub const RX_DATA_READ_POINTER: u16 = 0x28;

    pub const INTERRUPT_MASK: u16 = 0x2C;
}
