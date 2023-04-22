#![allow(clippy::inconsistent_digit_grouping, clippy::unusual_byte_groupings)]

use core::fmt;
use embedded_hal::blocking::spi::{Transfer, Write};
use embedded_hal::digital::v2::OutputPin;

use crate::bus::Bus;

const WRITE_MODE_MASK: u8 = 0b00000_1_00;

// TODO This name is not ideal, should be renamed to VDM
#[derive(Debug)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub struct FourWire<Spi: Transfer<u8> + Write<u8>, ChipSelect: OutputPin> {
    cs: ChipSelect,
    spi: Spi,
}

impl<Spi: Transfer<u8> + Write<u8>, ChipSelect: OutputPin> FourWire<Spi, ChipSelect> {
    pub fn new(spi: Spi, cs: ChipSelect) -> Self {
        Self { cs, spi }
    }

    pub fn release(self) -> (Spi, ChipSelect) {
        (self.spi, self.cs)
    }
}

impl<Spi: Transfer<u8> + Write<u8>, ChipSelect: OutputPin> Bus for FourWire<Spi, ChipSelect> {
    type Error =
        FourWireError<<Spi as Transfer<u8>>::Error, <Spi as Write<u8>>::Error, ChipSelect::Error>;

    fn read_frame(&mut self, block: u8, address: u16, data: &mut [u8]) -> Result<(), Self::Error> {
        let address_phase = address.to_be_bytes();
        let control_phase = block << 3;
        let data_phase = data;

        // set Chip select to Low, i.e. prepare to receive data
        self.cs.set_low().map_err(FourWireError::ChipSelectError)?;
        let result = (|| {
            self.spi
                .write(&address_phase)
                .and_then(|_| self.spi.write(&[control_phase]))
                .map_err(FourWireError::WriteError)?;
            self.spi
                .transfer(data_phase)
                .map_err(FourWireError::TransferError)?;
            Ok(())
        })();

        // set Chip select to High, i.e. we've finished listening
        self.cs.set_high().map_err(FourWireError::ChipSelectError)?;

        // then return the result of the transmission
        result
    }

    fn write_frame(&mut self, block: u8, address: u16, data: &[u8]) -> Result<(), Self::Error> {
        let address_phase = address.to_be_bytes();
        let control_phase = block << 3 | WRITE_MODE_MASK;
        let data_phase = data;

        // set Chip select to Low, i.e. prepare to transmit
        self.cs.set_low().map_err(FourWireError::ChipSelectError)?;
        let result = self
            .spi
            .write(&address_phase)
            .and_then(|_| self.spi.write(&[control_phase]))
            .and_then(|_| self.spi.write(data_phase))
            .map_err(FourWireError::WriteError);

        // set Chip select to High, i.e. we've finished transmitting
        self.cs.set_high().map_err(FourWireError::ChipSelectError)?;

        // then return the result of the transmission
        result
    }
}

// Must use map_err, ambiguity prevents From from being implemented
#[repr(u8)]
pub enum FourWireError<TransferError, WriteError, ChipSelectError> {
    TransferError(TransferError),
    WriteError(WriteError),
    ChipSelectError(ChipSelectError),
}

impl<TransferError, WriteError, ChipSelectError> fmt::Debug
    for FourWireError<TransferError, WriteError, ChipSelectError>
{
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "FourWireError::{}",
            match self {
                Self::TransferError(_) => "TransferError",
                Self::WriteError(_) => "WriteError",
                Self::ChipSelectError(_) => "ChipSelectError",
            }
        )
    }
}

// TODO Improved error rendering could be done with specialization.
// https://github.com/rust-lang/rust/issues/31844
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
