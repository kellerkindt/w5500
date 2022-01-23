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

# Example Usage

Below is a basic example of sending UDP packets to a remote host.  An important thing to confirm is the configuration
of the SPI implementation.  It must be set up to work as the W5500 chip requires.  That configuration is as follows:

* Data Order: Most significant bit first
* Clock Polarity: Idle low
* Clock Phase: Sample leading edge
* Clock speed: 33MHz maximum

Initialization and usage of owned `Device`:
```rust
    let mut spi = ...; // SPI interface to use
    let mut cs : OutputPin = ...; // chip select

    // alternative                     FourWireRef::new(&mut spi, &mut cs)
    let device = UninitializedDevice::new(FourWire::new(spi, cs))
            .initialize_manual(
                    MacAddress::new(0, 1, 2, 3, 4, 5),
                    Ipv4Addr::new(192, 168, 86, 79),
                    Mode::default()
            ).unwrap();

    let socket = device.socket();
    socket.connect(
        SocketAddr::new(IpAddr::V4(Ipv4Addr::new(192, 168, 86, 38)), 8000),
    ).unwrap();
    block!(interface.send(&mut socket, &[104, 101, 108, 108, 111, 10]));
    device.close(socket);

    // optional
    let (spi_bus, inactive_device) = device.deactivate();
```

Usage of borrowed SPI-Bus and previously initialized `Device`:
```rust
    let mut spi = ...; // SPI interface to use
    let mut cs: OutputPin = ...; // chip select

    let mut device: Option<InactiveDevice<..>> = ...; // maybe: previously initialized device
    let mut socket: Option<Socket> = ...; // maybe: previously opened socket
    
    if let (Some(socket), Some(device)) = (socket.as_mut(), device.as_mut()) {
        let mut buffer = [0u8; 1024];
        match device
            // scoped activation & usage of the SPI bus without move
            .activate_ref(FourWireRef::new(&mut spi, &mut cs))
            .receive(socket, &mut buffer)
        {
            Ok(..) => todo!(),
            Err(..) => todo!(),
        }
    }
```
## Todo

In no particular order, things to do to improve this driver.

* Add support for TCP server implementations
* Add support for DHCP
