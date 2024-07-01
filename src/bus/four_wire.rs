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
            Operation::TransferInPlace(data),
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
    use embedded_hal_mock::eh1::spi::{Mock as SpiMock, Transaction as SpiTransaction};

    use crate::{
        bus::{four_wire::WRITE_MODE_MASK, Bus},
        register,
    };

    use super::FourWire;

    #[test]
    fn test_read_frame() {
        let mut actual_version = [0_u8; 1];
        let mut expected_version = 5;

        let expectations = [
            SpiTransaction::transaction_start(),
            SpiTransaction::write_vec(register::common::VERSION.to_be_bytes().to_vec()),
            SpiTransaction::write(register::COMMON << 3),
            SpiTransaction::transfer_in_place(actual_version.to_vec(), vec![expected_version]),
            SpiTransaction::transaction_end(),
        ];

        let mock_spi = SpiMock::new(&expectations);

        let mut four_wire = FourWire::new(mock_spi);

        four_wire.read_frame(
            register::COMMON,
            register::common::VERSION,
            &mut actual_version,
        );

        four_wire.release().done();

        assert_eq!(expected_version, actual_version[0]);
    }

    #[test]
    fn test_write_frame() {
        let socket_0_reg = 0x01_u8;
        let socket_1_reg = 0x05_u8;
        let source_port = 49849_u16;

        let expectations = [
            SpiTransaction::transaction_start(),
            SpiTransaction::write_vec(register::socketn::SOURCE_PORT.to_be_bytes().to_vec()),
            SpiTransaction::write(socket_1_reg << 3 | WRITE_MODE_MASK),
            SpiTransaction::write_vec(source_port.to_be_bytes().to_vec()),
            SpiTransaction::transaction_end(),
        ];

        let mock_spi = SpiMock::new(&expectations);

        let mut four_wire = FourWire::new(mock_spi);

        four_wire.write_frame(
            socket_1_reg,
            register::socketn::SOURCE_PORT,
            &source_port.to_be_bytes(),
        );

        four_wire.release().done();
    }
}
