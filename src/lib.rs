#![cfg_attr(not(test), no_std)]
#![allow(unused)]
#![deny(rustdoc::broken_intra_doc_links)]
#![doc = include_str!("../README.md")]

pub mod bus;
mod device;
mod host;
pub mod net;
pub mod raw_device;
pub mod register;
mod socket;
pub mod tcp;
pub mod udp;
mod uninitialized_device;

pub use device::{Device, DeviceRefMut, InactiveDevice};
pub use host::{Dhcp, Host, HostConfig, Manual};
pub use net::MacAddress;
use register::common;
pub use uninitialized_device::{InitializeError, UninitializedDevice};

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

impl From<Mode> for common::Mode {
    fn from(value: Mode) -> Self {
        Self::Mode(value)
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
