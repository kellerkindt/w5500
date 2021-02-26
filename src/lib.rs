#![no_std]
#![allow(unused)]
#![deny(broken_intra_doc_links)]
#[macro_use(block)]

/// Settings for wake on LAN.  Allows the W5500 to optionally emit an interrupt upon receiving a packet
#[repr(u8)]
#[derive(Copy, Clone, PartialOrd, PartialEq)]
pub enum OnWakeOnLan {
    InvokeInterrupt = 0b00100000,
    Ignore = 0b00000000,
}

/// Settings for ping.  Allows the W5500 to respond to or ignore network ping requests
#[repr(u8)]
#[derive(Copy, Clone, PartialOrd, PartialEq)]
pub enum OnPingRequest {
    Respond = 0b00000000,
    Ignore = 0b00010000,
}

/// Use [TransmissionMode::PPoE] when talking
/// to an ADSL modem. Otherwise use [TransmissionMode::Ethernet]
#[repr(u8)]
#[derive(Copy, Clone, PartialOrd, PartialEq)]
pub enum ConnectionType {
    PPoE = 0b00001000,
    Ethernet = 0b00000000,
}

#[derive(Copy, Clone, PartialOrd, PartialEq)]
#[repr(u8)]
pub enum ArpResponses {
    Cache = 0b00000000,
    DropAfterUse = 0b00000010,
}

#[derive(Copy, Clone, PartialEq)]
pub struct Mode {
    on_wake_on_lan: OnWakeOnLan,
    on_ping_request: OnPingRequest,
    connection_type: ConnectionType,
    arp_responses: ArpResponses,
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

// TODO add better docs to all public items, add unit tests.

pub mod bus;
mod device;
mod host;
pub mod net;
pub mod register;
mod socket;
mod udp;
mod uninitialized_device;

pub use device::Device;
pub use host::{Dhcp, HostConfig, Manual};
pub use net::MacAddress;
pub use uninitialized_device::UninitializedDevice;
