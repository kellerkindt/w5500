# Example usage

Below some really basic usage how I am ca using it:

```rust
    let spi = ...; // SPI interface to use
    let cs : OutputPin = ...; // chip select
    
    let mut w5500 = W5500::new(spi, cs).unwrap();

    w5500.set_mode(false, false, false, false).unwrap();
    // using a 'locally administered' MAC address
    w5500.set_mac(&MacAddress::new(0x02, 0x01, 0x02, 0x03, 0x04, 0x05)).unwrap();
    w5500.set_ip(&IpAddress::new(192, 168, 0, 222)).unwrap();
    w5500.set_subnet(&IpAddress::new(255, 255, 255, 0)).unwrap();
    w5500.set_gateway(&IpAddress::new(192, 168, 0, 1)).unwrap();

    w5500.listen_udp(Socket::Socket1, 51).unwrap();
    let buffer = [u8; 2048];
    
    if let Some((ip, port, size)) = w5500.try_receive_udp(socket_rcv, &mut buffer).unwrap() {
        let (request_buffer, response_buffer) = buffer.split_mut_at(size);
        // ...

        w5500.send_udp(Socket::Socket0, 50, &ip, port, &response_buffer[..response_size]).unwrap();
    }
```
