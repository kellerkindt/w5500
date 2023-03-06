//! Networking data types.
//!
//! There may a standard for embedded networking in the future, see
//! [rust-embedded issue 348] and [RFC 2832]
//!
//! This is mostly ripped directly from [std::net].
//!
//! [rust-embedded issue 348]: https://github.com/rust-embedded/wg/issues/348
//! [std::net]: https://doc.rust-lang.org/std/net/index.html
//! [RFC 2832]: https://github.com/rust-lang/rfcs/pull/2832
#![deny(unsafe_code, missing_docs, warnings)]

// TODO remove some of these constructs and use equivalents available from embedded-nal

pub use embedded_nal::Ipv4Addr;

/// MAC address struct.  Can be instantiated with `MacAddress::new`.
///
/// This is an EUI-48 MAC address (previously called MAC-48).
#[derive(Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Debug, Hash, Default)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub struct MacAddress {
    /// Octets of the MAC address.
    pub octets: [u8; 6],
}

impl MacAddress {
    /// Creates a new EUI-48 MAC address from six eight-bit octets.
    ///
    /// The result will represent the EUI-48 MAC address
    /// `a`:`b`:`c`:`d`:`e`:`f`.
    ///
    /// # Examples
    ///
    /// ```
    /// use w5500::MacAddress;
    ///
    /// let addr = MacAddress::new(0x00, 0x00, 0x5E, 0x00, 0x00, 0x00);
    /// ```
    #[allow(clippy::many_single_char_names)]
    pub const fn new(a: u8, b: u8, c: u8, d: u8, e: u8, f: u8) -> MacAddress {
        MacAddress {
            octets: [a, b, c, d, e, f],
        }
    }

    /// Returns the six eight-bit integers that make up this address.
    ///
    /// # Examples
    ///
    /// ```
    /// use w5500::MacAddress;
    ///
    /// let addr = MacAddress::new(13, 12, 11, 10, 15, 14);
    /// assert_eq!([13u8, 12u8, 11u8, 10u8, 15u8, 14u8], addr.octets());
    /// ```
    pub const fn octets(&self) -> [u8; 6] {
        [
            self.octets[0],
            self.octets[1],
            self.octets[2],
            self.octets[3],
            self.octets[4],
            self.octets[5],
        ]
    }

    /// An EUI-48 MAC address representing an unspecified address:
    /// 00:00:00:00:00:00
    ///
    /// # Examples
    ///
    /// ```
    /// use w5500::MacAddress;
    ///
    /// let addr = MacAddress::UNSPECIFIED;
    /// assert_eq!(addr, MacAddress::new(0x00, 0x00, 0x00, 0x00, 0x00, 0x00));
    /// ```
    pub const UNSPECIFIED: Self = MacAddress::new(0, 0, 0, 0, 0, 0);
}

impl From<[u8; 6]> for MacAddress {
    /// Creates an `Ipv4Addr` from a six element byte array.
    ///
    /// # Examples
    ///
    /// ```
    /// use w5500::MacAddress;
    ///
    /// let addr = MacAddress::from([13u8, 12u8, 11u8, 10u8, 15u8, 14u8]);
    /// assert_eq!(MacAddress::new(13, 12, 11, 10, 15, 14), addr);
    /// ```
    fn from(octets: [u8; 6]) -> MacAddress {
        MacAddress::new(
            octets[0], octets[1], octets[2], octets[3], octets[4], octets[5],
        )
    }
}

impl ::core::fmt::Display for MacAddress {
    /// String formatter for MacAddress addresses.
    fn fmt(&self, fmt: &mut ::core::fmt::Formatter) -> ::core::fmt::Result {
        write!(
            fmt,
            "{:02X}:{:02X}:{:02X}:{:02X}:{:02X}:{:02X}",
            self.octets[0],
            self.octets[1],
            self.octets[2],
            self.octets[3],
            self.octets[4],
            self.octets[5],
        )
    }
}
