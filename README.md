# W5500 Driver

This crate is a driver for the WIZnet W5500 chip.  The W5500 chip is a hardwired TCP/IP embedded Ethernet controller
that enables embedded systems using SPI (Serial Peripheral Interface) to access the LAN. It is one of the
more popular ethernet modules on Arduino platforms.


[![Build Status](https://github.com/kellerkindt/w5500/workflows/Rust/badge.svg)](https://github.com/kellerkindt/w5500/actions?query=workflow%3ARust)
[![License](https://img.shields.io/badge/license-MIT%2FApache--2.0-blue.svg)](https://github.com/kellerkindt/w5500)
[![Crates.io](https://img.shields.io/crates/v/w5500.svg)](https://crates.io/crates/w5500)
[![Documentation](https://docs.rs/w5500/badge.svg)](https://docs.rs/w5500)
[![PRs Welcome](https://img.shields.io/badge/PRs-welcome-brightgreen.svg)](https://github.com/kellerkindt/w5500/issues/new)


## Embedded-HAL

Embedded-HAL is a standard set of traits meant to permit communication between MCU implementations and hardware drivers
like this one.  Any microcontroller that implements the
[`spi::FullDuplex<u8>`](https://docs.rs/embedded-hal/0.2.3/embedded_hal/spi/trait.FullDuplex.html) interface can use
this driver.

## Implementation

This driver is built in several layers of structs.

The lowest level (and the first a program would instantiate) is the `W5500` struct.  It contains a reference to the
chip-select [pin](https://docs.rs/embedded-hal/0.2.3/embedded_hal/digital/v2/trait.OutputPin.html).

The next layer is the `ActiveW5500` struct.  It contains a reference to a `W5500` instance, and an implementation of
the [`spi::FullDuplex<u8>`](https://docs.rs/embedded-hal/0.2.3/embedded_hal/spi/trait.FullDuplex.html) trait.  It has
the ability to actually communicate with the chip.  It has general methods for reading/writing to the chip, and
higher-level functions that can set up specific configuration, like the MAC address, etc.

The last layer is the network protocol.  Currently that is only `Udp`.  `Udp` is a tuple struct made up of an
`ActiveW5500` and a `Socket`.  This last layer can be used to send and receive UDP packets over the network via the
`receive` and `blocking_send` methods.

# Example Usage

Below is a basic example of listening for UDP packets and replying.  An important thing to confirm is the configuration
of the SPI implementation.  It must be set up to work as the W5500 chip requires.  That configuration is as follows:

* Data Order: Most significant bit first
* Clock Polarity: Idle low
* Clock Phase: Sample leading edge
* Clock speed: 33MHz maximum

```rust
    let mut spi = ...; // SPI interface to use
    let mut cs_w5500 : OutputPin = ...; // chip select
    
    let mut w5500 = W5500::with_initialisation(
        &mut cs_w5500, // borrowed for whole W5500 lifetime
        &mut spi, // borrowed for call to `with_initialisation` only
        OnWakeOnLan::Ignore,
        OnPingRequest::Respond,
        ConnectionType::Ethernet,
        ArpResponses::Cache,
    ).unwrap();
    
    let mut active = w5500.activate(&mut spi).unwrap();
    // using a 'locally administered' MAC address
    active.set_mac(MacAddress::new(0x02, 0x01, 0x02, 0x03, 0x04, 0x05)).unwrap();
    active.set_ip(IpAddress::new(192, 168, 0, 222)).unwrap();
    active.set_subnet(IpAddress::new(255, 255, 255, 0)).unwrap();
    active.set_gateway(IpAddress::new(192, 168, 0, 1)).unwrap();

    let socket0: UninitializedSocket = w5500.take_socket(Socket::Socket0).unwrap();
    let udp_server_socket = (&mut w5500, socket0).try_into_udp_server_socket(1234).unwrap();

    let mut buffer = [0u8; 256];
    let response = [104, 101, 108, 108, 111, 10];// "hello" as ASCII
    loop {
        if let Ok(Some((ip, port, len))) = udp_server_socket.receive(&mut buffer[..]) {
            udp_server_socket.blocking_send(ip, port, response[..]).unwrap();
        }
    }
```

## Todo

In no particular order, things to do to improve this driver.

* Add support for TCP
* Add support for DHCP
* Method to return socket back to the pool
* Make reset safe by requiring that all sockets be returned to the pool first
* Support a 3-wire SPI bus
* Sane defaults for IP/Gateway/Subnet
* Improve documentation
