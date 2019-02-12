# Example usage

Below some really basic usage how I am ca using it:

```rust
    let mut w5500: Option<W5500> = W5500::with_initialisation(
        &mut cs_w5500, // borrowed for whole W5500 lifetime
        &mut spi, // borrowed for call to `with_initialisation` only
        OnWakeOnLan::Ignore,
        OnPingRequest::Respond,
        ConnectionType::Ethernet,
        ArpResponses::Cache,
    )
    .ok();

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
