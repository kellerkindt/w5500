#![cfg_attr(not(test), no_std)]
#![allow(unused)]
#![deny(rustdoc::broken_intra_doc_links)]
#![doc = include_str!("../README.md")]

pub mod bus;
mod cursor;
mod device;
mod host;
pub mod net;
pub mod raw_device;
pub mod register;
mod socket;
pub mod tcp;
pub mod udp;
mod uninitialized_device;

#[doc(inline)]
pub use self::{
    device::{Device, DeviceState},
    host::{Dhcp, Host, HostConfig, Manual},
    net::MacAddress,
    uninitialized_device::{InitializeError, UninitializedDevice},
};

// TODO add better docs to all public items, add unit tests.

/// Settings for wake on LAN.  Allows the W5500 to optionally emit an interrupt upon receiving a packet
#[repr(u8)]
#[derive(Copy, Clone, Debug, PartialOrd, PartialEq, Eq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum OnWakeOnLan {
    InvokeInterrupt = 0b00100000,
    Ignore = 0b00000000,
}

/// Ping Block Mode
///
/// Settings for ping.  Allows the W5500 to respond to or ignore network ping requests
#[repr(u8)]
#[derive(Copy, Clone, Debug, PartialOrd, PartialEq, Eq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum OnPingRequest {
    /// 0: Disable Ping block
    Respond = 0b00000000,
    /// 1 : Enable Ping block
    ///
    /// If the bit is ‘1’, it blocks the response to a ping request.
    Ignore = 0b00010000,
}

/// Use [ConnectionType::PPoE] when talking
/// to an ADSL modem. Otherwise use [ConnectionType::Ethernet]
#[repr(u8)]
#[derive(Copy, Clone, Debug, PartialOrd, PartialEq, Eq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum ConnectionType {
    PPoE = 0b00001000,
    Ethernet = 0b00000000,
}

/// Force ARP
///
#[derive(Copy, Clone, Debug, PartialOrd, PartialEq, Eq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
#[repr(u8)]
pub enum ArpResponses {
    /// 0 : Disable Force ARP mode
    Cache = 0b00000000,
    /// 1 : Enable Force ARP mode
    ///
    /// In Force ARP mode, It forces on sending ARP Request whenever data is
    /// sent.
    DropAfterUse = 0b00000010,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub struct Mode {
    pub on_wake_on_lan: OnWakeOnLan,
    pub on_ping_request: OnPingRequest,
    pub connection_type: ConnectionType,
    pub arp_responses: ArpResponses,
}

impl Mode {
    pub fn to_register(self) -> [u8; 1] {
        [self.to_u8()]
    }

    pub fn to_u8(self) -> u8 {
        let mut register = 0;
        register |= self.on_wake_on_lan as u8;
        register |= self.on_ping_request as u8;
        register |= self.connection_type as u8;
        register |= self.arp_responses as u8;

        register
    }
}

impl Default for Mode {
    fn default() -> Self {
        Self {
            on_wake_on_lan: OnWakeOnLan::Ignore,
            on_ping_request: OnPingRequest::Respond,
            connection_type: ConnectionType::Ethernet,
            arp_responses: ArpResponses::DropAfterUse,
        }
    }
}

#[cfg(test)]
mod test {
    use crate::Mode;

    #[test]
    fn test_mode_register() {
        let ping_respond_and_force_arp = Mode {
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

        let all_enabled = Mode {
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
