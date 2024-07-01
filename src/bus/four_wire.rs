#![allow(clippy::inconsistent_digit_grouping, clippy::unusual_byte_groupings)]

use core::fmt;
use embedded_hal::spi::{ErrorType, Operation, SpiDevice};

use crate::bus::Bus;

const WRITE_MODE_MASK: u8 = 0b00000_1_00;

// TODO This name is not ideal, should be renamed to VDM
#[derive(Debug)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub struct FourWire<SPI> {
    spi: SPI,
}

impl<SPI> FourWire<SPI> {
    pub fn new(spi: SPI) -> Self {
        Self { spi }
    }

    pub fn release(self) -> SPI {
        self.spi
    }
}

impl<SPI: SpiDevice> Bus for FourWire<SPI> {
    type Error = <SPI as ErrorType>::Error;

    fn read_frame(&mut self, block: u8, address: u16, data: &mut [u8]) -> Result<(), SPI::Error> {
        let address_phase = address.to_be_bytes();
        let control_phase = block << 3;

        self.spi.transaction(&mut [
            Operation::Write(&address_phase),
            Operation::Write(&[control_phase]),
            Operation::Write(data),
        ])?;

        Ok(())
    }

    fn write_frame(&mut self, block: u8, address: u16, data: &[u8]) -> Result<(), SPI::Error> {
        let control_phase = block << 3 | WRITE_MODE_MASK;

        let address_phase = address.to_be_bytes();
        self.spi.transaction(&mut [
            Operation::Write(&address_phase),
            Operation::Write(&[control_phase]),
            Operation::Write(data),
        ])?;

        Ok(())
    }
}

#[cfg(test)]
mod test {
    use embedded_hal::digital::v2::OutputPin;
    use embedded_hal_mock::{
        pin::{Mock as PinMock, State as PinState, Transaction as PinTransaction},
        spi::{Mock as SpiMock, Transaction as SpiTransaction},
    };

    use crate::{
        bus::{four_wire::WRITE_MODE_MASK, Bus},
        register,
    };

    use super::FourWire;

    #[test]
    fn test_read_frame() {
        let mut cs_pin = PinMock::new(&[
            // we begin with pin HIGH
            PinTransaction::set(PinState::High),
            // When reading
            PinTransaction::set(PinState::Low),
            // When finished reading
            PinTransaction::set(PinState::High),
        ]);

        // initiate the pin to high.
        cs_pin.set_high().expect("Should set pin to high");

        let mut actual_version = [0_u8; 1];
        let mut expected_version = 5;

        let expectations = [
            SpiTransaction::write(register::common::VERSION.to_be_bytes().to_vec()),
            SpiTransaction::write(vec![register::COMMON << 3]),
            SpiTransaction::transfer(actual_version.to_vec(), vec![expected_version]),
        ];

        let mock_spi = SpiMock::new(&expectations);

        let mut four_wire = FourWire::new(mock_spi, cs_pin);

        four_wire.read_frame(
            register::COMMON,
            register::common::VERSION,
            &mut actual_version,
        );

        assert_eq!(expected_version, actual_version[0]);
    }

    #[test]
    fn test_write_frame() {
        let mut cs_pin = PinMock::new(&[
            // we begin with pin HIGH
            PinTransaction::set(PinState::High),
            // When reading
            PinTransaction::set(PinState::Low),
            // When finished reading
            PinTransaction::set(PinState::High),
        ]);

        // initiate the pin to high.
        cs_pin.set_high().expect("Should set pin to high");

        let socket_0_reg = 0x01_u8;
        let socket_1_reg = 0x05_u8;
        let source_port = 49849_u16;

        let expectations = [
            SpiTransaction::write(register::socketn::SOURCE_PORT.to_be_bytes().to_vec()),
            SpiTransaction::write(vec![socket_1_reg << 3 | WRITE_MODE_MASK]),
            SpiTransaction::write(source_port.to_be_bytes().to_vec()),
        ];

        let mock_spi = SpiMock::new(&expectations);

        let mut four_wire = FourWire::new(mock_spi, cs_pin);

        four_wire.write_frame(
            socket_1_reg,
            register::socketn::SOURCE_PORT,
            &source_port.to_be_bytes(),
        );
    }
}
