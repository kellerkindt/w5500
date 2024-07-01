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

The [`embedded-hal`](https://docs.rs/embedded-hal/latest/embedded_hal/index.html) is a standard set
of traits meant to permit communication between MCU implementations and hardware drivers like this
one.  Any microcontroller that implements the
[`spi::SpiDevice`](https://docs.rs/embedded-hal/latest/embedded_hal/spi/trait.SpiDevice.html) or
[`spi::SpiBus`](https://docs.rs/embedded-hal/latest/embedded_hal/spi/trait.SpiBus.html) can use this
driver.

# Example Usage

Below is a basic example of sending UDP packets to a remote host.  An important thing to confirm is the configuration
of the SPI implementation.  It must be set up to work as the W5500 chip requires.  That configuration is as follows:

* Data Order: Most significant bit first
* Clock Polarity: Idle low
* Clock Phase: Sample leading edge
* Clock speed: 33MHz maximum

```rust,no_run
use embedded_nal::{IpAddr, Ipv4Addr, SocketAddr};
#
# struct Mock;
#
# impl embedded_hal::spi::ErrorType for Mock {
#     type Error = core::convert::Infallible;
# }
#
# impl embedded_hal::spi::SpiDevice for Mock {
#     fn transaction(&mut self, operations: &mut [embedded_hal::spi::Operation<'_, u8>]) -> Result<(), Self::Error> {
#         Ok(())
#     }
# }
use embedded_nal::UdpClientStack;

let mut spi = Mock;

let mut device = w5500::UninitializedDevice::new(w5500::bus::FourWire::new(spi))
        .initialize_manual(
                w5500::MacAddress::new(0, 1, 2, 3, 4, 5),
                Ipv4Addr::new(192, 168, 86, 79),
                w5500::Mode::default()
        ).unwrap();

// Allocate a UDP socket to send data with
let mut socket = device.socket().unwrap();

// Connect the socket to the IP address and port we want to send to.
device.connect(&mut socket,
    SocketAddr::new(IpAddr::V4(Ipv4Addr::new(192, 168, 86, 38)), 8000),
).unwrap();

// Send the data
nb::block!(device.send(&mut socket, &[104, 101, 108, 108, 111, 10]));

// Optionally close the socket
device.close(socket);
```
## Todo

In no particular order, things to do to improve this driver.

* Add support for TCP server implementations
* Add support for DHCP
