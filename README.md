# Example usage

Below some really basic usage how I am ca using it:

```rust
    let mut spi = ...; // SPI interface to use
    let mut cs_w5500 : OutputPin = ...; // chip select
    
    let mut w5500: Option<W5500> = W5500::with_initialisation(
        &mut cs_w5500, // borrowed for whole W5500 lifetime
        &mut spi, // borrowed for call to `with_initialisation` only
        OnWakeOnLan::Ignore,
        OnPingRequest::Respond,
        ConnectionType::Ethernet,
        ArpResponses::Cache,
    )
    .ok();
    
    if let Some(ref mut w5500) = w5500 {
        let mut w5500: ActiveW5500<_> = w5500.activate(&mut spi).unwrap();
        // using a 'locally administered' MAC address
        active.set_mac(MacAddress::new(0x02, 0x01, 0x02, 0x03, 0x04, 0x05)).unwrap();
        active.set_ip(IpAddress::new(192, 168, 0, 222)).unwrap();
        active.set_subnet(IpAddress::new(255, 255, 255, 0)).unwrap();
        active.set_gateway(IpAddress::new(192, 168, 0, 1)).unwrap();
    }

    let mut udp_server_socket: Option<UdpSocket> = w5500.as_mut().and_then(|w5500| {
        let mut w5500: ActiveW5500<_> = w5500.activate(&mut spi).ok()?;
        let socket0: UninitializedSocket = w5500.take_socket(Socket::Socket0)?;
        (&mut w5500, socket0).try_into_udp_server_socket(1234).ok()
    });

    let mut buffer = [0u8; 256];
    if let (Some(ref mut w5500), Some(ref socket)) = (
        w5500.as_mut().and_then(w5500.activate(&mut spi).ok()),
        udp_server_socket,
    ) {
        if let Ok(Some((ip, port, len))) = (w5500, socket).receive(&mut buffer[..]) {
            let (request_buffer, response_buffer) = buffer.split_mut_at(len);

            // ... fill the response_buffer with some data ...

            (w5500, socket).blocking_send(ip, port, response_buffer[..response_len]).unwrap();
        }
    }
```
