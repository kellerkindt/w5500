#![no_std]
#![allow(unused)]
#![deny(broken_intra_doc_links)]

#[macro_use(block)]
extern crate nb;

/// MAC address struct.  Represents a MAC address as a u8 array of length 6. Can be instantiated with `MacAddress::new`
#[derive(Copy, Clone, PartialOrd, PartialEq, Default, Debug)]
pub struct MacAddress {
    pub address: [u8; 6],
}

impl MacAddress {
    pub fn new(n0: u8, n1: u8, n2: u8, n3: u8, n4: u8, n5: u8) -> Self {
        MacAddress {
            address: [n0, n1, n2, n3, n4, n5],
        }
    }
}

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

pub mod bus;
mod device;
mod host;
mod inactive_device;
pub mod net;
pub mod register;
mod socket;
mod udp;
pub mod uninitialized_device;

pub use net::MacAddress;
