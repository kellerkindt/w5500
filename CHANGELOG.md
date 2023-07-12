# Changelog
All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Added

- Add `defmt` features for enabling `defmt::Format` to most structs and errors by [@elpiel](https://github.com/elpiel) ([#39](https://github.com/kellerkindt/w5500/issues/39))
- Fixed an issue where internal function names were conflicting with trait names by [@ryan-summers](https://github.com/ryan-summers) ([#36](https://github.com/kellerkindt/w5500/issues/36))

### Fixed

- Fixed an issue that caused corruption when reading partial MACRAW frames by [@Felix-El](https://github.com/Felix-El) ([#47](https://github.com/kellerkindt/w5500/pull/47))

## [0.4.1] - January 2nd, 2023

### Added

- Fix indexing for phy configuration register by [@Wassasin](https://github.com/Wassasin) ([#32](https://github.com/kellerkindt/w5500/issues/32))
- Add support for MACRAW operation mode by [@ryan-summers](https://github.com/ryan-summers) ([#33](https://github.com/kellerkindt/w5500/issues/33))

## [0.4.0] - January 22nd, 2022

### Added
- Add support for 3-wire SPI bus ([#15](https://github.com/kellerkindt/w5500/issues/15))
- Add constructors for network types ([#21](https://github.com/kellerkindt/w5500/issues/21))
- Add method to change PHY configuration ([#23](https://github.com/kellerkindt/w5500/issues/23))
- Add feature `no-chip-version-assertion` for compatible chips with unexpected version information
- Add `MacAddress::octets()`
- Add `impl From<[u8; 6]> for MacAddress`
- Add `Device::deactivate` and `InactiveDevice::activate`
- Add re-export of `FourWireError` and `ThreeWireError`
- Add `FourWireRef` to be able to use `Device` with borrowed SPI and CS
- Add `DeviceRefMut` to be able to use `Device` without moving ownership
- Add getter: `Device::{gateway,subnet_mask,mac,ip,version}`
- Restructure and implement embedded-nal UDP trais ([#26](https://github.com/kellerkindt/w5500/issues/26)) - big thanks to [@jonahd-g](https://github.com/jonahd-g)
- Add TCP client support ([#24](https://github.com/kellerkindt/w5500/issues/24)) - big thanks to [@ryan-summers](https://github.com/ryan-summers)

### Changed
- Updated dependencies ([#22](https://github.com/kellerkindt/w5500/issues/22))
- `Mode` fields are now publicly acessible

### Removed
- Cargo.lock ([#20](https://github.com/kellerkindt/w5500/issues/20))
- Replace `net::Ipv4Addr` with `embedded_nal::Ipv4Addr`

## [0.3.0] - June 10, 2020

### Breaking changes
- Require [`v2::OutputPins`](https://github.com/rust-embedded/embedded-hal/blob/9e6ab5a1ee8900830bd4fe56f0a84ddb0bccda3f/src/digital/v2.rs)
- [`OutputPin` is now taken by ownership](https://github.com/kellerkindt/w5500/blob/d02bbf7e5cc837e658671d1467305523136376cc/src/lib.rs#L131) instead of mut refs [#13](https://github.com/kellerkindt/w5500/issues/13)

### Changes
- Upgrade to Rust 2018 Edition
- Many doc updates, thanks @jonahbron 
