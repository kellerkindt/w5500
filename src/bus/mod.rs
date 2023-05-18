use core::fmt::Debug;

mod four_wire;
mod four_wire_ref;
mod three_wire;

use crate::register::{
    self,
    common::{RetryCount, RetryTime},
};

pub use self::four_wire::FourWire;
pub use self::four_wire::FourWireError;
pub use self::four_wire_ref::FourWireRef;
pub use self::four_wire_ref::OutputPinRef;
pub use self::four_wire_ref::SpiRef;
pub use self::three_wire::ThreeWire;
pub use self::three_wire::ThreeWireError;

pub trait Bus {
    type Error: Debug;

    fn read_frame(&mut self, block: u8, address: u16, data: &mut [u8]) -> Result<(), Self::Error>;

    fn write_frame(&mut self, block: u8, address: u16, data: &[u8]) -> Result<(), Self::Error>;

    /// Reset the device
    #[inline]
    fn reset(&mut self) -> Result<(), Self::Error> {
        self.write_frame(
            register::COMMON,
            register::common::MODE,
            &register::common::Mode::Reset.to_register(),
        )?;

        Ok(())
    }

    #[inline]
    fn set_mode(&mut self, mode_options: register::common::Mode) -> Result<(), Self::Error> {
        self.write_frame(
            register::COMMON,
            register::common::MODE,
            &mode_options.to_register(),
        )?;
        Ok(())
    }

    #[inline]
    fn version(&mut self) -> Result<u8, Self::Error> {
        let mut version_register = [0_u8];
        self.read_frame(
            register::COMMON,
            register::common::VERSION,
            &mut version_register,
        )?;

        Ok(version_register[0])
    }

    /// RTR (Retry Time-value Register) [R/W] [0x0019 – 0x001A] [0x07D0]
    ///
    /// # Example
    /// ```
    /// use w5500::register::common::RetryTime;
    ///
    /// let default = RetryTime::from_millis(200);
    /// assert_eq!(RetryTime::default(), default);
    ///
    /// // E.g. 4000 (register) = 400ms
    /// let four_hundred_ms = RetryTime::from_millis(400);
    /// assert_eq!(four_hundred_ms.to_u16(), 4000);
    /// ```
    #[inline]
    fn set_retry_timeout(&mut self, retry_time_value: RetryTime) -> Result<(), Self::Error> {
        self.write_frame(
            register::COMMON,
            register::common::RETRY_TIME,
            &retry_time_value.to_register(),
        )?;

        Ok(())
    }

    /// RTR (Retry Time-value Register) [R/W] [0x0019 – 0x001A] [0x07D0]
    ///
    /// E.g. 4000 = 400ms
    #[inline]
    fn current_retry_timeout(&mut self) -> Result<RetryTime, Self::Error> {
        let mut retry_time_register: [u8; 2] = [0, 0];
        self.read_frame(
            register::COMMON,
            register::common::RETRY_TIME,
            &mut retry_time_register,
        )?;

        Ok(RetryTime::from_register(retry_time_register))
    }

    /// Set a new value for the Retry Count register.
    ///
    /// RCR (Retry Count Register) [R/W] [0x001B] [0x08]
    fn set_retry_count(&mut self, retry_count: RetryCount) -> Result<(), Self::Error> {
        self.write_frame(
            register::COMMON,
            register::common::RETRY_COUNT,
            &retry_count.to_register(),
        )?;

        Ok(())
    }

    /// Get the current Retry Count value
    /// RCR (Retry Count Register) [R/W] [0x001B] [0x08]
    ///
    /// E.g. In case of errors it will retry for 7 times:
    /// `RCR = 0x0007`
    #[inline]
    fn current_retry_count(&mut self) -> Result<RetryCount, Self::Error> {
        let mut retry_count_register: [u8; 1] = [0];
        self.read_frame(
            register::COMMON,
            register::common::RETRY_COUNT,
            &mut retry_count_register,
        )?;

        Ok(RetryCount::from_register(retry_count_register))
    }
}

pub struct BusRef<'a, B: Bus>(pub &'a mut B);

impl<B: Bus> Bus for BusRef<'_, B> {
    type Error = B::Error;

    #[inline]
    fn read_frame(&mut self, block: u8, address: u16, data: &mut [u8]) -> Result<(), Self::Error> {
        self.0.read_frame(block, address, data)
    }

    #[inline]
    fn write_frame(&mut self, block: u8, address: u16, data: &[u8]) -> Result<(), Self::Error> {
        self.0.write_frame(block, address, data)
    }
}
